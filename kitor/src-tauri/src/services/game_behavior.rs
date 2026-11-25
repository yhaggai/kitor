//! Reviewed catalog behavior shared by detection, installation and launching.
//! Most games use declarative defaults; adapters are for format-specific work.
use super::registry_service::Platform;
pub use super::registry_service::{
    BundledAdapter, InstallTarget, LaunchMethod, LaunchPreparation, LaunchRoot,
    LaunchWorkingDirectory, NativePreparation,
};
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

pub use super::game_detection::Detection;

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
    pub validate_on_import: bool,
    pub commands: std::collections::BTreeMap<Platform, NativeCommand>,
    pub subdirectory: Option<String>,
    pub args: Vec<String>,
    pub runtime: Option<String>,
    pub runtime_data: Option<String>,
    pub preparation: Option<LaunchPreparation>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCommand {
    #[serde(default)]
    pub method: LaunchMethod,
    #[serde(default)]
    pub root: LaunchRoot,
    pub bundle_name: Option<String>,
    #[serde(default)]
    pub working_directory: LaunchWorkingDirectory,
    #[serde(default)]
    pub executable_files: Vec<String>,
    #[serde(default)]
    pub required_directories: Vec<String>,
    #[serde(default)]
    pub signatures: Vec<LaunchSignature>,
    pub preparation: Option<NativePreparation>,
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub required_files: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchSignature {
    pub path: String,
    pub prefix: Vec<u8>,
}

impl NativeCommand {
    pub fn validate_for_platform(&self, platform: Platform) -> Result<(), String> {
        self.validate()?;
        if self.root == LaunchRoot::AppBundle && platform != Platform::MacOS {
            return Err("App-bundle roots require macOS".into());
        }
        if self.preparation == Some(NativePreparation::GrimHebrewPreferences)
            && platform != Platform::Windows
        {
            return Err("Grim Fandango preference preparation requires Windows".into());
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_relative(&self.program)?;
        let bundle = self.root == LaunchRoot::AppBundle;
        if bundle != self.bundle_name.is_some() {
            return Err(
                "App-bundle roots require a bundle name; game roots cannot specify one".into(),
            );
        }
        if let Some(name) = &self.bundle_name {
            validate_relative(name)?;
            if name.contains('/') || !name.to_ascii_lowercase().ends_with(".app") {
                return Err("Bundle name must be a single .app directory name".into());
            }
        }
        if !bundle
            && (self.method == LaunchMethod::AppBundle
                || self.working_directory != LaunchWorkingDirectory::Root)
        {
            return Err("Bundle launch and working directories require an app-bundle root".into());
        }
        if self.method == LaunchMethod::AppBundle && !self.env.is_empty() {
            return Err("Use executable launch when environment variables are required".into());
        }
        for path in self
            .required_files
            .iter()
            .chain(&self.required_directories)
            .chain(&self.executable_files)
        {
            validate_relative(path)?;
        }
        for signature in &self.signatures {
            validate_relative(&signature.path)?;
            if signature.prefix.is_empty() || signature.prefix.len() > 4096 {
                return Err("Launch signatures must contain 1 to 4096 bytes".into());
            }
        }
        Ok(())
    }
}

impl LaunchProfile {
    pub fn is_native(&self) -> bool {
        !self.commands.is_empty()
    }
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
        self.detection.validate()?;
        if self.bundled.is_some() && self.bundled_resource.is_some() {
            return Err("Choose one bundled package resource or adapter".into());
        }
        if let Some(profiles) = &self.launch {
            for profile in [&profiles.original, &profiles.patched]
                .into_iter()
                .chain(profiles.original_macos.iter())
            {
                for path in profile
                    .subdirectory
                    .iter()
                    .chain(profile.runtime.iter())
                    .chain(profile.runtime_data.iter())
                {
                    validate_relative(path)?;
                }
                if profile.validate_on_import && profile.commands.is_empty() {
                    return Err("Import validation requires native commands".into());
                }
                for (platform, command) in &profile.commands {
                    command.validate_for_platform(*platform)?;
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
                super::games::drifter_patch::macos_target(
                    &std::env::home_dir().ok_or("Cannot locate home directory")?,
                )
            }
            _ => Ok(root.to_path_buf()),
        }
    }
    pub fn validate_install_target(&self, root: &Path) -> Result<PathBuf, String> {
        if matches!(self.target, InstallTarget::DrifterUserData) && cfg!(target_os = "macos") {
            super::games::drifter_patch::validate_macos_build(root)?;
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
                        profiles
                            .original_macos
                            .as_ref()
                            .unwrap_or(&profiles.original)
                            .clone()
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
