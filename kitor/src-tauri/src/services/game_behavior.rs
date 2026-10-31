//! Reviewed catalog behavior shared by detection, installation and launching.
//! Most games use declarative defaults; adapters are for format-specific work.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GameBehavior {
    pub detection: Detection,
    pub bundled: Option<BundledAdapter>,
    pub bundled_resource: Option<String>,
    pub target: InstallTarget,
    pub launch: Option<LaunchProfiles>,
    pub library_editions: Vec<String>,
    pub health_files: Vec<String>,
    pub manual_patch_manifest: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Detection {
    pub adapter: DetectionAdapter,
    pub priority: i32,
    pub layouts: Vec<Layout>,
    pub extra_subdirectories: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub root: String,
    pub required_files: Vec<String>,
    #[serde(default)]
    pub data: String,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionAdapter {
    #[default]
    Patterns,
    Tentacle,
    TentacleRemastered,
    Owlboy,
    MonkeyNative,
    SpaceQuest,
    Toonstruck,
    Torin,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundledAdapter {
    Tentacle,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallTarget {
    #[default]
    GameDirectory,
    DrifterUserData,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchProfiles {
    pub original: LaunchProfile,
    #[serde(default)]
    pub original_macos: Option<LaunchProfile>,
    pub patched: LaunchProfile,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LaunchProfile {
    pub native: Option<NativeAdapter>,
    pub commands: std::collections::BTreeMap<String, NativeCommand>,
    pub native_game_id: Option<String>,
    pub subdirectory: Option<String>,
    pub args: Vec<String>,
    pub runtime: Option<String>,
    pub runtime_data: Option<String>,
    pub preparation: Option<LaunchPreparation>,
    pub prepare_hebrew_preferences: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommand {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub required_files: Vec<String>,
}
impl LaunchProfile {
    pub fn is_native(&self) -> bool {
        self.native.is_some() || !self.commands.is_empty()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeAdapter {
    Drifter,
    MonkeyNative,
    Grim,
    TentacleRemastered,
    Owlboy,
    Undertale,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchPreparation {
    KyrandiaSignature,
    TentacleClassic,
}

pub fn for_game(id: &str) -> Result<GameBehavior, String> {
    Ok(super::registry_service::load_registry()?
        .games
        .into_iter()
        .find(|g| g.scummvm_id == id)
        .map(|g| g.behavior)
        .unwrap_or_default())
}

fn validate_relative(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.contains(['\\', ':'])
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(format!("Invalid relative behavior path: {path}"));
    }
    Ok(())
}

impl GameBehavior {
    pub fn validate(&self) -> Result<(), String> {
        if self.bundled.is_some() && self.bundled_resource.is_some() {
            return Err("Choose one bundled package resource or adapter".into());
        }
        if let Some(profiles) = &self.launch {
            for profile in [&profiles.original, &profiles.patched].into_iter().chain(profiles.original_macos.iter()) {
                if profile.native.is_some() && !profile.commands.is_empty() {
                    return Err("Choose native commands or a native adapter, not both".into());
                }
                for path in profile
                    .subdirectory
                    .iter()
                    .chain(profile.runtime.iter())
                    .chain(profile.runtime_data.iter())
                {
                    validate_relative(path)?;
                }
                for command in profile.commands.values() {
                    validate_relative(&command.program)?;
                    for name in &command.required_files {
                        validate_relative(name)?;
                    }
                }
            }
        }
        for path in self
            .health_files
            .iter()
            .chain(self.manual_patch_manifest.iter())
        {
            validate_relative(path)?;
        }
        Ok(())
    }
    pub fn install_target(&self, root: &Path) -> Result<PathBuf, String> {
        match self.target {
            InstallTarget::DrifterUserData if cfg!(target_os = "macos") => {
                super::drifter_patch::macos_target(
                    &std::env::home_dir().ok_or("Cannot locate home directory")?,
                )
            }
            _ => Ok(root.to_path_buf()),
        }
    }
    pub fn validate_install_target(&self, root: &Path) -> Result<PathBuf, String> {
        if matches!(self.target, InstallTarget::DrifterUserData) && cfg!(target_os = "macos") {
            super::drifter_patch::validate_macos_build(root)?;
        }
        self.install_target(root)
    }
    pub fn launch_profile(&self, patched: bool) -> LaunchProfile {
        self.launch
            .as_ref()
            .map(|profiles| {
                if patched {
                    profiles.patched.clone()
                } else {
                    if cfg!(target_os = "macos") {
                        profiles.original_macos.as_ref().unwrap_or(&profiles.original).clone()
                    } else {
                        profiles.original.clone()
                    }
                }
            })
            .unwrap_or_else(|| LaunchProfile {
                args: vec!["--auto-detect".into()],
                ..Default::default()
            })
    }
}

#[cfg(test)]
#[path = "game_behavior/tests.rs"]
mod tests;
