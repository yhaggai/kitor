//! Generic, evidence-based detection. Game identities and fingerprints are registry data.
use super::patch_lifecycle::safe_path;
use super::registry_service::{
    DetectionDigest, DetectionEdition, DetectionMode, DetectionPathStyle, GameDefinition,
    TextNormalization,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Detection {
    pub mode: DetectionMode,
    pub priority: i32,
    pub layouts: Vec<Layout>,
    pub extra_subdirectories: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    #[serde(default)]
    pub path_style: DetectionPathStyle,
    pub root: String,
    #[serde(default)]
    pub ancestor: u8,
    pub selected_name: Option<String>,
    #[serde(default)]
    pub data: String,
    pub edition: Option<DetectionEdition>,
    pub required_files: Vec<String>,
    #[serde(default)]
    pub required_directories: Vec<String>,
    #[serde(default)]
    pub file_checks: Vec<FileCheck>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FileCheck {
    Text {
        path: String,
        equals: String,
        #[serde(default)]
        normalization: TextNormalization,
    },
    Prefix {
        path: String,
        bytes: Vec<u8>,
        min_size: Option<u64>,
    },
    Digest {
        path: String,
        algorithm: DetectionDigest,
        bytes: Option<u64>,
        size: Option<u64>,
        values: Vec<String>,
    },
    Size {
        path: String,
        min: u64,
    },
}
fn relative(path: &str, allow_root: bool) -> Result<(), String> {
    if allow_root && (path.is_empty() || path == ".") {
        return Ok(());
    }
    if path.is_empty()
        || path.contains(['\\', ':'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("Unsafe detection path: {path}"));
    }
    Ok(())
}
impl Detection {
    pub fn validate(&self) -> Result<(), String> {
        if self.mode == DetectionMode::Layouts && self.layouts.is_empty() {
            return Err("Layout detection requires rules".into());
        }
        for path in &self.extra_subdirectories {
            relative(path, false)?;
        }
        for layout in &self.layouts {
            relative(&layout.root, true)?;
            relative(&layout.data, true)?;
            if layout.ancestor > 3 || layout.required_files.is_empty() {
                return Err(
                    "Detection layouts need required files and at most three ancestors".into(),
                );
            }
            if let Some(name) = &layout.selected_name {
                relative(name, false)?;
                if name.contains('/') {
                    return Err("Selected name must be a single directory name".into());
                }
            }
            for path in layout
                .required_files
                .iter()
                .chain(&layout.required_directories)
            {
                relative(path, false)?;
            }
            for check in &layout.file_checks {
                relative(check.path(), false)?;
                match check {
                    FileCheck::Prefix { bytes, .. } if bytes.is_empty() || bytes.len() > 4096 => {
                        return Err("Detection prefixes need 1 to 4096 bytes".into())
                    }
                    FileCheck::Digest {
                        algorithm,
                        values,
                        bytes,
                        ..
                    } => {
                        let length = if *algorithm == DetectionDigest::Sha256 {
                            64
                        } else {
                            32
                        };
                        if *bytes == Some(0)
                            || values.is_empty()
                            || values.iter().any(|v| {
                                v.len() != length || !v.bytes().all(|b| b.is_ascii_hexdigit())
                            })
                        {
                            return Err("Invalid detection digest".into());
                        }
                    }
                    FileCheck::Text { equals, .. } if equals.len() > 1048576 => {
                        return Err("Detection text is too large".into())
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
impl FileCheck {
    fn path(&self) -> &str {
        match self {
            Self::Text { path, .. }
            | Self::Prefix { path, .. }
            | Self::Digest { path, .. }
            | Self::Size { path, .. } => path,
        }
    }
    fn matches(&self, root: &Path) -> Option<bool> {
        let path = safe_path(root, self.path()).ok()?;
        if !path.is_file() {
            return Some(false);
        }
        let mut file = fs::File::open(path).ok()?;
        let size = file.metadata().ok()?.len();
        Some(match self {
            Self::Text {
                equals,
                normalization,
                ..
            } => {
                if size > 1048576 {
                    return Some(false);
                }
                let mut text = String::new();
                file.read_to_string(&mut text).ok()?;
                match normalization {
                    TextNormalization::Exact => text == *equals,
                    TextNormalization::TrimCrLf => text.replace('\r', "").trim() == equals,
                }
            }
            Self::Prefix {
                bytes, min_size, ..
            } => {
                if min_size.is_some_and(|min| size < min) {
                    return Some(false);
                }
                let mut actual = vec![0; bytes.len()];
                file.read_exact(&mut actual).is_ok() && actual == *bytes
            }
            Self::Digest {
                algorithm,
                bytes,
                size: expected,
                values,
                ..
            } => {
                if expected.is_some_and(|expected| expected != size) {
                    return Some(false);
                }
                let reader = file.take(bytes.unwrap_or(u64::MAX));
                let digest = match algorithm {
                    DetectionDigest::Sha256 => digest::<sha2::Sha256>(reader)?,
                    DetectionDigest::Md5 => digest::<md5::Md5>(reader)?,
                };
                values
                    .iter()
                    .any(|value| value.eq_ignore_ascii_case(&digest))
            }
            Self::Size { min, .. } => size >= *min,
        })
    }
}
fn digest<D: Digest>(mut reader: impl Read) -> Option<String> {
    let mut hash = D::new();
    let mut buffer = [0; 65536];
    loop {
        let read = reader.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Some(
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}
fn resolve(root: &Path, relative: &str) -> Option<PathBuf> {
    if relative.is_empty() || relative == "." {
        return root.is_dir().then(|| root.to_path_buf());
    }
    safe_path(root, relative).ok()
}
fn layout_root(layout: &Layout, input: &Path) -> Option<PathBuf> {
    if let Some(name) = &layout.selected_name {
        if !input.file_name()?.eq_ignore_ascii_case(name) {
            return None;
        }
    }
    let base = input.ancestors().nth(layout.ancestor as usize)?;
    let root = resolve(base, &layout.root)?;
    if !layout
        .required_files
        .iter()
        .all(|name| safe_path(&root, name).is_ok_and(|p| p.is_file()))
        || !layout
            .required_directories
            .iter()
            .all(|name| safe_path(&root, name).is_ok_and(|p| p.is_dir()))
        || !layout
            .file_checks
            .iter()
            .all(|check| check.matches(&root) == Some(true))
    {
        return None;
    }
    let data = resolve(&root, &layout.data)?;
    if !data.is_dir() {
        return None;
    }
    match layout.path_style {
        DetectionPathStyle::Canonical => data.canonicalize().ok(),
        DetectionPathStyle::Preserve => Some(data),
    }
}
fn candidates(def: &GameDefinition, path: &Path) -> Vec<PathBuf> {
    let mut roots = vec![path.to_path_buf()];
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
    {
        for name in ["Contents/Resources/game/game", "Contents/Resources/game"] {
            if let Some(root) = resolve(path, name) {
                roots.push(root);
            }
        }
    }
    let bases = roots.clone();
    for base in bases {
        for name in def
            .behavior
            .detection
            .extra_subdirectories
            .iter()
            .chain(def.detection_patterns.detection_subdirectory.iter())
        {
            if let Some(root) = resolve(&base, name) {
                roots.push(root);
            }
        }
    }
    roots
}
pub fn detect(def: &GameDefinition, path: &Path) -> Option<PathBuf> {
    detect_edition(def, path, None)
}
fn detect_edition(
    def: &GameDefinition,
    path: &Path,
    edition: Option<DetectionEdition>,
) -> Option<PathBuf> {
    let detection = &def.behavior.detection;
    detection.validate().ok()?;
    let roots = candidates(def, path);
    if detection.mode == DetectionMode::Layouts || !detection.layouts.is_empty() {
        // Evidence rules are authoritative: a failed fingerprint never falls back to a common filename.
        for layout in &detection.layouts {
            if edition.is_some() && edition != layout.edition {
                continue;
            }
            for root in &roots {
                if let Some(data) = layout_root(layout, root) {
                    return Some(data);
                }
            }
        }
        return None;
    }
    if edition.is_some() {
        return None;
    }
    roots.into_iter().find(|root| {
        def.detection_patterns
            .unique_files
            .iter()
            .any(|name| safe_path(root, name).is_ok_and(|path| path.is_file()))
    })
}
/// Compatibility entry point for format-specific package tools, using registry rules.
pub fn detect_registered(
    id: &str,
    path: &Path,
    edition: Option<DetectionEdition>,
) -> Option<PathBuf> {
    let registry = super::registry_service::load_registry().ok()?;
    let def = registry.games.iter().find(|game| game.scummvm_id == id)?;
    detect_edition(def, path, edition)
}

#[cfg(test)]
#[path = "game_detection/tests.rs"]
mod tests;
