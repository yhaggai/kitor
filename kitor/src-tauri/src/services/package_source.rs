//! Stage verified translation packages before touching game files.
use super::{game_behavior::GameBehavior, registry_service::PatchDefinition};
use std::{fs, path::Path};

pub enum PackageSource {
    Download,
    OwlboyMacSteam,
}
impl PackageSource {
    pub fn resolve(patch: &PatchDefinition, _behavior: &GameBehavior) -> Result<Self, String> {
        if cfg!(target_os = "macos")
            && patch.version == "0.2.0-beta.2-kitor2-download1"
            && patch.filename.as_deref() == Some("owlboy-he-0.2.0-beta.2-kitor2-download1.zip")
        {
            return Ok(Self::OwlboyMacSteam);
        }
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
        let bundled;
        let archive = if matches!(self, Self::OwlboyMacSteam) {
            if id != "owlboy" || _language != "he" {
                return Err("Unexpected Owlboy patch identity".into());
            }
            const EXPECTED_EXE: &str = "3b177abee8d2bf4bde0ba13ff3c7355459a91fe7f8bf978c1f373f2a500bcc5d";
            const PATCHED_EXE: &str = "0c8ebae73783145ac45701d13a6f409979e8f1d4a28e209df6e83f7fcb1ece69";
            let executable = super::patch_lifecycle::hash(&target.join("Owlboy.exe"))?;
            if executable != EXPECTED_EXE && executable != PATCHED_EXE {
                return Err("This Hebrew patch requires the Steam macOS Owlboy build 138-4809.".into());
            }
            bundled = tempfile::Builder::new().suffix(".zip").tempfile().map_err(|e| e.to_string())?;
            fs::write(bundled.path(), include_bytes!("../../../../patches-workspace/owlboy/he/steam-138-4809/patch.zip"))
                .map_err(|e| e.to_string())?;
            bundled.path()
        } else {
            archive
        };
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
        if super::games::tentacle::detect_classic(target).is_some() { "classic" } else { "remastered" }
    } else { "default" };
    let selected = root.join("_platforms").join(platform).join(edition);
    if !selected.join("patch.json").is_file() {
        return Err("Downloaded patch does not support this platform or edition".into());
    }
    Ok(selected)
}

#[cfg(test)]
mod tests;
