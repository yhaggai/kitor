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
    pub title_he: Option<String>,
    /// Catalog keywords such as `classic` or `detective`. A game may have several.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Platforms>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gog: Option<GogInfo>,
    /// GOG checkout link that adds this game, or the pack that sells it, to the cart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_to_cart_url: Option<String>,
    pub detection_patterns: DetectionPatterns,
    pub patches: Vec<PatchDefinition>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub discover_without_patch: bool,
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

fn registry_content(development_path: &Path) -> Result<String, String> {
    if cfg!(debug_assertions) {
        match std::fs::read_to_string(development_path) {
            Ok(content) => return Ok(content),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(error) => return Err(format!("Registry load error: {error}")),
        }
    }
    Ok(include_str!("../../../patches/registry.json").to_owned())
}

pub fn load_registry() -> Result<Registry, String> {
    // Production trust starts with the compiled catalog. A working-directory
    // file must never replace package URLs, checksums, or launch metadata.
    let content = registry_content(Path::new("../patches/registry.json"))?;

    println!("Registry file loaded successfully, parsing JSON...");

    let mut registry: Registry = serde_json::from_str(&content).map_err(|e| {
        let err_msg = format!("Failed to parse registry JSON: {}", e);
        println!("Registry parse error: {}", err_msg);
        err_msg
    })?;

    println!(
        "Registry loaded: {} games, {} total patches",
        registry.games.len(),
        registry
            .games
            .iter()
            .map(|g| g.patches.len())
            .sum::<usize>()
    );

    // Older external registries predate behavior profiles. Use the reviewed
    // built-in behavior for known identities instead of stale saved arguments.
    let bundled: Registry = serde_json::from_str(include_str!("../../../patches/registry.json"))
        .map_err(|e| format!("Invalid bundled registry: {e}"))?;
    migrate_bundled_packages(&mut registry, &bundled);
    prepare_behaviors(&mut registry, &bundled)?;
    updates::merge_cached(&mut registry);
    #[cfg(debug_assertions)]
    if let Some(path) = super::games::disco_elysium::preview_archive()? {
        if let Some(game) = registry.games.iter_mut().find(|g| g.scummvm_id == "disco-elysium") {
            game.patches = vec![super::games::disco_elysium::preview_definition(&path)?];
        }
    }
    Ok(registry)
}

// Migrate saved catalogs from embedded patches to the reviewed download entry.
// Unknown legacy entries are removed because this build contains no payloads.
fn migrate_bundled_packages(registry: &mut Registry, bundled: &Registry) {
    for game in &mut registry.games {
        let built = bundled
            .games
            .iter()
            .find(|g| g.scummvm_id == game.scummvm_id);
        game.patches.retain_mut(|patch| {
            if patch.source_type != "bundled" {
                return true;
            }
            let Some(def) = built else { return false };
            let Some(packaged) = def
                .patches
                .iter()
                .find(|p| p.source_type != "bundled" && p.language == patch.language)
                .or_else(|| {
                    (def.behavior.bundled.is_some() || def.behavior.bundled_resource.is_some())
                        .then(|| def.patches.iter().find(|p| p.language == patch.language))
                        .flatten()
                })
            else {
                return false;
            };
            *patch = packaged.clone();
            game.behavior.bundled = def.behavior.bundled;
            game.behavior.bundled_resource = def.behavior.bundled_resource.clone();
            true
        });
    }
}

fn prepare_behaviors(registry: &mut Registry, bundled: &Registry) -> Result<(), String> {
    for game in &mut registry.games {
        if game.behavior.launch.is_none() {
            if let Some(def) = bundled
                .games
                .iter()
                .find(|g| g.scummvm_id == game.scummvm_id)
            {
                game.behavior = def.behavior.clone();
            } else if let Some(legacy) = &game.launch_config {
                let patched = super::game_behavior::LaunchProfile {
                    args: legacy.args.clone(),
                    subdirectory: legacy.subdirectory.clone(),
                    ..Default::default()
                };
                let original = super::game_behavior::LaunchProfile {
                    args: legacy
                        .args
                        .iter()
                        .filter(|a| !a.starts_with("--language="))
                        .cloned()
                        .collect(),
                    ..Default::default()
                };
                game.behavior.launch = Some(super::game_behavior::LaunchProfiles {
                    original,
                    patched,
                    original_macos: None,
                });
            }
        }
        game.behavior.validate()?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "registry_service/tests.rs"]
mod tests;
