//! Format-specific adapters selected by reviewed catalog configuration.
use super::game_behavior::{BundledAdapter, DetectionAdapter, LaunchProfile, NativeAdapter};
use super::registry_service::GameDefinition;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn stage(
    adapter: BundledAdapter,
    destination: &Path,
    _id: &str,
    language: &str,
    version: &str,
    target: &Path,
) -> Result<(), String> {
    match adapter {
        BundledAdapter::Tentacle => super::tentacle::stage(destination, language, version, target),
    }
}

pub fn detect(def: &GameDefinition, path: &Path) -> Option<PathBuf> {
    let detection = &def.behavior.detection;
    match detection.adapter {
        DetectionAdapter::Tentacle => return super::tentacle::detect(path),
        DetectionAdapter::TentacleRemastered => return super::tentacle::detect_remastered(path),
        DetectionAdapter::Owlboy => return super::owlboy::detect(path),
        DetectionAdapter::MonkeyNative => {
            return super::monkey_native::detect(path, &def.scummvm_id)
        }
        _ => {}
    }
    for layout in &detection.layouts {
        let root = path.join(&layout.root);
        if !layout.required_files.is_empty()
            && layout
                .required_files
                .iter()
                .all(|name| root.join(name).is_file())
        {
            return root.join(&layout.data).canonicalize().ok();
        }
    }
    let mut candidates = vec![path.to_path_buf()];
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
    {
        candidates.extend([
            path.join("Contents/Resources/game/game"),
            path.join("Contents/Resources/game"),
        ]);
    }
    for candidate in candidates {
        let mut roots = vec![candidate.clone()];
        roots.extend(
            detection
                .extra_subdirectories
                .iter()
                .map(|p| candidate.join(p)),
        );
        if let Some(subdir) = &def.detection_patterns.detection_subdirectory {
            roots.push(candidate.join(subdir));
        }
        for root in roots {
            let matches = match detection.adapter {
                DetectionAdapter::SpaceQuest => is_space_quest(&root, &def.scummvm_id),
                DetectionAdapter::Toonstruck => is_toonstruck(&root),
                DetectionAdapter::Torin => is_torin(&root),
                _ => std::fs::read_dir(&root).ok().is_some_and(|entries| {
                    entries.flatten().any(|entry| {
                        entry.file_type().is_ok_and(|t| t.is_file())
                            && def.detection_patterns.unique_files.iter().any(|name| {
                                entry
                                    .file_name()
                                    .to_string_lossy()
                                    .eq_ignore_ascii_case(name)
                            })
                    })
                }),
            };
            if matches {
                return Some(root);
            }
        }
    }
    None
}

pub fn native_command(
    profile: &LaunchProfile,
    root: &Path,
    game_id: &str,
) -> Result<Option<Command>, String> {
    if !profile.commands.is_empty() {
        let config = profile
            .commands
            .get(super::patch_lifecycle::platform())
            .ok_or("Native launch is not supported on this platform")?;
        let program = super::patch_lifecycle::safe_path(root, &config.program)?;
        if !program.is_file() {
            return Err("Native game executable not found".into());
        }
        for name in &config.required_files {
            if !super::patch_lifecycle::safe_path(root, name)?.is_file() {
                return Err(format!("Required game file not found: {name}"));
            }
        }
        let mut command = Command::new(program);
        command
            .current_dir(root)
            .args(&config.args)
            .envs(&config.env);
        return Ok(Some(command));
    }
    let Some(adapter) = profile.native else {
        return Ok(None);
    };
    let command = match adapter {
        NativeAdapter::Drifter => drifter_launch_command(root)?,
        NativeAdapter::MonkeyNative => super::monkey_native::launch_command(
            root,
            profile.native_game_id.as_deref().unwrap_or(game_id),
        )?,
        NativeAdapter::Grim => {
            let command = super::grim_remastered::launch_command(root)?;
            if profile.prepare_hebrew_preferences {
                super::grim_remastered::prepare_launch(root)?;
            }
            command
        }
        NativeAdapter::TentacleRemastered => super::tentacle_remastered::launch_command(root)?,
        NativeAdapter::Owlboy => super::owlboy::launch_command(root)?,
        NativeAdapter::Undertale => undertale_command(root)?,
    };
    Ok(Some(command))
}

pub fn drifter_launch_command(install_path: &Path) -> Result<Command, String> {
    if !cfg!(target_os = "macos") {
        return Err("The Drifter native launcher currently supports macOS only".into());
    }
    let bundle = [
        install_path.to_path_buf(),
        install_path.join("TheDrifter.app"),
    ]
    .into_iter()
    .find(|path| {
        path.join("Contents/MacOS/The Drifter").is_file()
            && path
                .join("Contents/Resources/Data/globalgamemanagers")
                .is_file()
    })
    .ok_or("The Drifter app was not found in its installation folder")?;
    let bundle = bundle.canonicalize().map_err(|e| e.to_string())?;
    let mut cmd = Command::new("/usr/bin/open");
    cmd.arg(&bundle)
        .current_dir(bundle.parent().ok_or("Invalid Drifter app path")?);
    Ok(cmd)
}
fn undertale_command(data: &Path) -> Result<Command, String> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let bundle = data
            .parent()
            .and_then(Path::parent)
            .ok_or("Invalid UNDERTALE app path")?;
        if !bundle.join("Contents/MacOS/Mac_Runner").is_file() {
            return Err("UNDERTALE Mac runner not found".into());
        }
        let mut cmd = Command::new("open");
        cmd.arg(bundle);
        cmd
    };
    #[cfg(target_os = "windows")]
    let mut cmd = Command::new(data.join("UNDERTALE.exe"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return Err("UNDERTALE native launch supports macOS and Windows only".into());
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        cmd.current_dir(data);
        Ok(cmd)
    }
}

pub(super) fn is_space_quest(path: &Path, id: &str) -> bool {
    use sha2::Digest;
    #[derive(serde::Deserialize)]
    struct Signature {
        id: String,
        signature: String,
        sha256: Vec<String>,
        required: Vec<String>,
    }
    let signatures: Vec<Signature> =
        serde_json::from_str(include_str!("../../data/space_quest_detection.json"))
            .expect("bundled Space Quest signatures");
    let Some(signature) = signatures.iter().find(|s| s.id == id) else {
        return false;
    };
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };
    let files: std::collections::HashMap<_, _> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().to_ascii_lowercase(),
                entry.path(),
            )
        })
        .collect();
    if !signature
        .required
        .iter()
        .all(|name| files.contains_key(&name.to_ascii_lowercase()))
    {
        return false;
    }
    let Some(file) = files.get(&signature.signature.to_ascii_lowercase()) else {
        return false;
    };
    let Ok(bytes) = std::fs::read(file) else {
        return false;
    };
    signature
        .sha256
        .contains(&format!("{:x}", sha2::Sha256::digest(bytes)))
}

// GOG keeps Toonstruck's identifying resources below MISC. Match both files
// case-insensitively, but retain the parent containing ACT1/ACT2 for ScummVM.
pub(super) fn is_toonstruck(path: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };
    entries.flatten().any(|entry| {
        if !entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case("misc")
        {
            return false;
        }
        let Ok(files) = std::fs::read_dir(entry.path()) else {
            return false;
        };
        let names: std::collections::HashSet<_> = files
            .flatten()
            .filter(|file| file.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|file| file.file_name().to_string_lossy().to_ascii_lowercase())
            .collect();
        names.contains("toonfont.caf") && names.contains("toonstrk.res")
    })
}

// Some GOG Book One bundles also contain FATE.PAK. Together with the Hebrew
// font it wins ScummVM's detection as Book Two. Detect using only Book One's
// signature, then load all remaining assets through --extrapath.
pub(super) fn prepare_kyrandia_launch_path(
    data_path: &Path,
    launch_path: &Path,
) -> Result<PathBuf, String> {
    let signature = std::fs::read_dir(data_path)
        .map_err(|e| format!("Failed to read Kyrandia data: {}", e))?
        .flatten()
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("GEMCUT.PAK")
                && entry.path().is_file()
        })
        .ok_or("Kyrandia GEMCUT.PAK not found")?;
    std::fs::create_dir_all(launch_path)
        .map_err(|e| format!("Failed to create Kyrandia launch folder: {}", e))?;
    // Refresh after installing or restoring a patch; the signature changes.
    std::fs::copy(signature.path(), launch_path.join("GEMCUT.PAK"))
        .map_err(|e| format!("Failed to prepare Kyrandia launch: {}", e))?;
    Ok(launch_path.to_path_buf())
}

pub fn scummvm_paths(
    profile: &LaunchProfile,
    root: &Path,
    staging: &Path,
) -> Result<(PathBuf, Option<PathBuf>), String> {
    use super::game_behavior::LaunchPreparation;
    let use_root = matches!(
        profile.preparation,
        Some(LaunchPreparation::TentacleClassic)
    ) && super::tentacle::detect_classic(root).is_some();
    let data = if use_root {
        root.to_path_buf()
    } else {
        profile
            .subdirectory
            .as_ref()
            .map(|p| super::patch_lifecycle::safe_path(root, p))
            .transpose()?
            .unwrap_or(root.to_path_buf())
    };
    match profile.preparation {
        Some(LaunchPreparation::KyrandiaSignature) => {
            Ok((prepare_kyrandia_launch_path(&data, staging)?, Some(data)))
        }
        _ => Ok((data, None)),
    }
}

// English GOG/CD signatures from ScummVM's SCI detection tables. ScummVM
// fingerprints the first 5000 bytes; file sizes distinguish demos and editions.
fn is_torin(root: &Path) -> bool {
    use md5::{Digest, Md5};
    use std::io::Read;
    let Ok(entries) = std::fs::read_dir(root) else { return false; };
    let files: std::collections::HashMap<_, _> = entries.flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| (entry.file_name().to_string_lossy().to_ascii_lowercase(), entry.path())).collect();
    [
        ("resmap.000", 9799, "bb3b0b22ff08df54fbe2d06263409be6"),
        ("ressci.000", 55973887, "693a259d346c9360f4a0c11fdaae430a"),
    ].iter().all(|(name, size, hash)| {
        let Some(path) = files.get(*name) else { return false; };
        let Ok(file) = std::fs::File::open(path) else { return false; };
        if !file.metadata().is_ok_and(|meta| meta.len() == *size) { return false; }
        let mut prefix = Vec::new();
        file.take(5000).read_to_end(&mut prefix).is_ok() && format!("{:x}", Md5::digest(prefix)) == *hash
    })
}
