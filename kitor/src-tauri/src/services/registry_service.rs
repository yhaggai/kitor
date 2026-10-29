use serde::{Deserialize, Serialize};
use std::path::Path;

#[path = "patch_catalog.rs"]
pub mod updates;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub games: Vec<GameDefinition>,
    pub metadata: RegistryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GogInstaller {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GogInfo {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installers: Option<Vec<GogInstaller>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platforms {
    #[serde(default)]
    pub windows: bool,
    #[serde(default)]
    pub osx: bool,
    #[serde(default)]
    pub linux: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDefinition {
    #[serde(default)]
    pub behavior: super::game_behavior::GameBehavior,
    pub scummvm_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steam_app_id: Option<u32>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Platforms>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gog: Option<GogInfo>,
    pub detection_patterns: DetectionPatterns,
    pub patches: Vec<PatchDefinition>,
    // Read-only compatibility with registries created before behavior profiles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_config: Option<LaunchConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_guide: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launcher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdirectory: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPatterns {
    pub unique_files: Vec<String>,
    pub file_patterns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detection_subdirectory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchDefinition {
    #[serde(default)]
    pub release_revision: u64,
    #[serde(default)]
    pub install_manifest: Option<String>,
    pub language: String,
    pub version: String,
    pub author: String,
    pub download_url: String,
    pub source_type: String,
    pub checksum: Option<String>,
    pub file_size: Option<i64>,
    pub description: String,
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install_script: Option<PostInstallScript>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInstallScript {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub osx: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linux: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    pub version: String,
    pub last_updated: String,
    pub source: String,
}

pub fn load_registry() -> Result<Registry, String> {
    // Get the registry file path relative to the executable
    let registry_path = if cfg!(debug_assertions) {
        // In development, look in the workspace root (one level up from src-tauri)
        Path::new("../patches/registry.json")
    } else {
        // In production, look relative to the executable
        Path::new("patches/registry.json")
    };

    println!("Attempting to load registry from: {:?}", registry_path);
    
    let content = std::fs::read_to_string(registry_path)
        .or_else(|error| {
            // Finder-launched bundles have no project working directory.
            // Keep the reviewed registry available without relying on cwd.
            if error.kind() == std::io::ErrorKind::NotFound {
                Ok(include_str!("../../../patches/registry.json").to_owned())
            } else { Err(error) }
        })
        .map_err(|e| {
            let err_msg = format!("Failed to read registry file: {}", e);
            println!("Registry load error: {}", err_msg);
            err_msg
        })?;

    println!("Registry file loaded successfully, parsing JSON...");
    
    let mut registry: Registry = serde_json::from_str(&content)
        .map_err(|e| {
            let err_msg = format!("Failed to parse registry JSON: {}", e);
            println!("Registry parse error: {}", err_msg);
            err_msg
        })?;

    println!("Registry loaded: {} games, {} total patches", 
        registry.games.len(),
        registry.games.iter().map(|g| g.patches.len()).sum::<usize>()
    );

    // Older external registries predate behavior profiles. Use the reviewed
    // built-in behavior for known identities instead of stale saved arguments.
    let bundled: Registry = serde_json::from_str(include_str!("../../../patches/registry.json"))
        .map_err(|e| format!("Invalid bundled registry: {e}"))?;
    prepare_behaviors(&mut registry, &bundled)?;
    updates::merge_cached(&mut registry);
    Ok(registry)
}


fn prepare_behaviors(registry: &mut Registry, bundled: &Registry) -> Result<(), String> {
    for game in &mut registry.games {
        if game.behavior.launch.is_none() {
            if let Some(def) = bundled.games.iter().find(|g| g.scummvm_id == game.scummvm_id) {
                game.behavior = def.behavior.clone();
            } else if let Some(legacy) = &game.launch_config {
                let patched = super::game_behavior::LaunchProfile { args: legacy.args.clone(), subdirectory: legacy.subdirectory.clone(), ..Default::default() };
                let original = super::game_behavior::LaunchProfile { args: legacy.args.iter().filter(|a| !a.starts_with("--language=")).cloned().collect(), ..Default::default() };
                game.behavior.launch = Some(super::game_behavior::LaunchProfiles { original, patched });
            }
        }
        game.behavior.validate()?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "registry_service/tests.rs"]
mod tests;
