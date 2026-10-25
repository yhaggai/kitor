use serde::{Deserialize, Serialize};
use std::path::Path;

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
    pub scummvm_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Platforms>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gog: Option<GogInfo>,
    pub detection_patterns: DetectionPatterns,
    pub patches: Vec<PatchDefinition>,
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
        .map_err(|e| {
            let err_msg = format!("Failed to read registry file: {}", e);
            println!("Registry load error: {}", err_msg);
            err_msg
        })?;

    println!("Registry file loaded successfully, parsing JSON...");
    
    let registry: Registry = serde_json::from_str(&content)
        .map_err(|e| {
            let err_msg = format!("Failed to parse registry JSON: {}", e);
            println!("Registry parse error: {}", err_msg);
            err_msg
        })?;

    println!("Registry loaded: {} games, {} total patches", 
        registry.games.len(),
        registry.games.iter().map(|g| g.patches.len()).sum::<usize>()
    );

    Ok(registry)
}

