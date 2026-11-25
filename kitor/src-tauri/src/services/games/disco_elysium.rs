//! Private Final Cut preview metadata; detection is configured in the registry.
#[cfg(any(test, debug_assertions))]
use std::{
    fs,
    path::{Path, PathBuf},
};
#[cfg(test)]
#[path = "disco_elysium/tests.rs"]
mod tests;
/// Explicit opt-in for private testing; never compiled into release builds.
#[cfg(debug_assertions)]
pub fn preview_archive() -> Result<Option<PathBuf>, String> {
    let Some(path) = std::env::var_os("KITOR_DISCO_PREVIEW") else {
        return Ok(None);
    };
    let path = PathBuf::from(path)
        .canonicalize()
        .map_err(|e| format!("Local Disco preview: {e}"))?;
    if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("zip") {
        return Err("KITOR_DISCO_PREVIEW must point to a local ZIP".into());
    }
    Ok(Some(path))
}
#[cfg(debug_assertions)]
pub fn preview_definition(
    path: &Path,
) -> Result<super::super::registry_service::PatchDefinition, String> {
    use super::super::{patch_lifecycle, registry_service::PatchDefinition};
    Ok(PatchDefinition {
        supported_builds: Vec::new(),
        match_files: Default::default(),
        release_revision:0,install_manifest:Some("patch.json".into()),language:"he".into(),
        version:"0.1.0-preview1".into(),author:"Kitor local preview".into(),
        download_url:"local-preview://disco-elysium".into(),source_type:crate::services::registry_service::SourceType::LocalPreview,
        checksum:Some(patch_lifecycle::hash(path)?),file_size:Some(fs::metadata(path).map_err(|e|e.to_string())?.len() as i64),
        description:"Private Hebrew preview (4,165 entries). Select Arabic in-game. Untranslated text remains Arabic.".into(),
        filename:Some("Disco-Elysium-Hebrew-0.1.0-preview1.zip".into()),post_install_script:None,
    })
}
