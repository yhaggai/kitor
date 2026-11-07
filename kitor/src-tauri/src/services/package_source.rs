//! Translation bytes always arrive through the verified download path.
use super::{game_behavior::GameBehavior, registry_service::PatchDefinition};
use std::{fs, path::Path};

pub enum PackageSource {
    Download,
}
impl PackageSource {
    pub fn resolve(patch: &PatchDefinition, _behavior: &GameBehavior) -> Result<Self, String> {
        if patch.source_type == "bundled" || !patch.download_url.starts_with("https://") {
            return Err("Patch has no downloadable package; refresh the catalog".into());
        }
        Ok(Self::Download)
    }

    pub fn stage(
        &self,
        destination: &Path,
        archive: &Path,
        id: &str,
        _language: &str,
        _version: &str,
        target: &Path,
    ) -> Result<(), String> {
        super::file_service::validate_zip(archive)?;
        // Extract beside the destination so moving the selected platform works
        // on Windows and cannot cross filesystems. No game files are touched.
        let extracted = tempfile::Builder::new().prefix("kitor-extracted-")
            .tempdir_in(destination.parent().ok_or("Missing staging parent")?)
            .map_err(|e| e.to_string())?;
        super::file_service::extract_zip(archive, extracted.path())?;
        let root = selected_root(extracted.path(), id, target, super::patch_lifecycle::platform())?;
        for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            fs::rename(entry.path(), destination.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn selected_root(root: &Path, id: &str, target: &Path, platform: &str) -> Result<std::path::PathBuf, String> {
    if !root.join("_platforms").exists() {
        return Ok(root.to_path_buf());
    }
    if root.join("patch.json").exists() {
        return Err("Ambiguous download package layout".into());
    }
    let edition = if id == "tentacle" {
        if super::tentacle::detect_classic(target).is_some() { "classic" } else { "remastered" }
    } else { "default" };
    let selected = root.join("_platforms").join(platform).join(edition);
    if !selected.join("patch.json").is_file() {
        return Err("Downloaded patch does not support this platform or edition".into());
    }
    Ok(selected)
}

#[cfg(test)]
mod tests;
