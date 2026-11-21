//! Stage verified translation packages before touching game files.
use super::{game_behavior::GameBehavior, registry_service::PatchDefinition};
use std::{fs, path::Path};

pub enum PackageSource {
    Download,
    #[cfg(debug_assertions)]
    LocalPreview(std::path::PathBuf),
    OwlboyMacSteam,
    OwlboyWindowsSteam,
    SpaceQuest4Scummvm,
}
impl PackageSource {
    pub fn resolve_for_target(
        patch: &PatchDefinition,
        behavior: &GameBehavior,
        id: &str,
        target: &Path,
    ) -> Result<Self, String> {
        if cfg!(target_os = "windows")
            && id == "owlboy"
            && patch.version == "0.2.0-beta.2-kitor2-download1"
            && patch.language == "he"
            && patch.release_revision == 1
            && super::patch_lifecycle::hash(&target.join("Owlboy.exe"))
                .is_ok_and(|hash| owlboy_windows_138_4809(&hash))
        {
            return Ok(Self::OwlboyWindowsSteam);
        }
        Self::resolve(patch, behavior)
    }

    pub fn download_metadata(&self, patch: &PatchDefinition) -> Result<(String, Option<String>), String> {
        let key = match self {
            Self::OwlboyMacSteam => Some("owlboy-macos-steam-138-4809"),
            Self::OwlboyWindowsSteam => Some("owlboy-windows-steam-138-4809"),
            _ => None,
        };
        if let Some(key) = key {
            let entries: serde_json::Value = serde_json::from_str(include_str!("../../../patches/compat-downloads.json"))
                .map_err(|e| e.to_string())?;
            let entry = &entries[key];
            let url = entry["download_url"].as_str().ok_or("Missing compatibility download URL")?;
            let checksum = entry["checksum"].as_str().ok_or("Missing compatibility download checksum")?;
            return Ok((url.into(), Some(checksum.into())));
        }
        Ok((patch.download_url.clone(), patch.checksum.clone()))
    }

    pub fn resolve(patch: &PatchDefinition, _behavior: &GameBehavior) -> Result<Self, String> {
        if cfg!(target_os = "macos")
            && patch.version == "0.2.0-beta.2-kitor2-download1"
            && patch.filename.as_deref() == Some("owlboy-he-0.2.0-beta.2-kitor2-download1.zip")
        {
            return Ok(Self::OwlboyMacSteam);
        }
        if cfg!(target_os = "macos")
            && patch.version == "0.1.0-download1"
            && patch.filename.as_deref() == Some("sq4-he-0.1.0-download1.zip")
            && patch.checksum.as_deref() == Some("98149fc0bcfb14671e356bad7544074e9510f1e5319fddab45dd8a045f2534fb")
        {
            return Ok(Self::SpaceQuest4Scummvm);
        }
        if patch.source_type == "bundled" {
            return Err("Bundled patches are no longer supported; refresh the catalog".into());
        }
        #[cfg(debug_assertions)]
        if patch.source_type == "local-preview" && patch.download_url == "local-preview://disco-elysium" {
            let path = super::games::disco_elysium::preview_archive()?.ok_or("Local preview is not enabled")?;
            if patch.checksum.as_deref() != Some(super::patch_lifecycle::hash(&path)?.as_str()) {
                return Err("Local preview changed; refresh before installing".into());
            }
            return Ok(Self::LocalPreview(path));
        }
        if !patch.download_url.starts_with("https://") {
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
        if matches!(self, Self::OwlboyWindowsSteam) {
            if id != "owlboy" || _language != "he" {
                return Err("Unexpected Owlboy patch identity".into());
            }
            let executable = super::patch_lifecycle::hash(&target.join("Owlboy.exe"))?;
            if !owlboy_windows_138_4809(&executable) {
                return Err("The Owlboy executable changed before staging the patch.".into());
            }
        } else if matches!(self, Self::OwlboyMacSteam) {
            if id != "owlboy" || _language != "he" {
                return Err("Unexpected Owlboy patch identity".into());
            }
            const EXPECTED_EXE: &str = "3b177abee8d2bf4bde0ba13ff3c7355459a91fe7f8bf978c1f373f2a500bcc5d";
            const PATCHED_EXE: &str = "0c8ebae73783145ac45701d13a6f409979e8f1d4a28e209df6e83f7fcb1ece69";
            let executable = super::patch_lifecycle::hash(&target.join("Owlboy.exe"))?;
            if executable != EXPECTED_EXE && executable != PATCHED_EXE {
                return Err("This Hebrew patch requires the Steam macOS Owlboy build 138-4809.".into());
            }
        }
        super::file_service::validate_zip(archive)?;
        // Extract beside the destination so moving the selected platform works
        // on Windows and cannot cross filesystems. No game files are touched.
        let extracted = tempfile::Builder::new().prefix("kitor-extracted-")
            .tempdir_in(destination.parent().ok_or("Missing staging parent")?)
            .map_err(|e| e.to_string())?;
        super::file_service::extract_zip(archive, extracted.path())?;
        let resource_only = matches!(self, Self::SpaceQuest4Scummvm);
        let root = selected_root(extracted.path(), id, target,
            if resource_only { "windows" } else { super::patch_lifecycle::platform() })?;
        if resource_only {
            use super::patch_lifecycle::{Package, Operation, Source};
            let manifest = root.join("patch.json");
            let mut package: Package = serde_json::from_slice(&fs::read(&manifest).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            if id != "sq4" || package.scummvm_id != id || package.language != _language || package.version != _version {
                return Err("Unexpected Space Quest IV patch identity".into());
            }
            let plan = package.platforms.get("windows").ok_or("Missing ScummVM resource plan")?.clone();
            if plan.files.is_empty() || plan.files.iter().any(|file|
                file.operation != Operation::Install || !matches!(file.source, Some(Source::Copy { .. }))) {
                return Err("Space Quest IV package is not a resource-only patch".into());
            }
            package.platforms.insert("osx".into(), plan);
            fs::write(manifest, serde_json::to_vec(&package).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        }
        if matches!(self, Self::OwlboyMacSteam) {
            select_owlboy_catalog(&root, target)?;
        }
        for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            fs::rename(entry.path(), destination.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn owlboy_windows_138_4809(executable: &str) -> bool {
    matches!(executable,
        "919e3a7db7d2255d743133335e521c7c95fc57ead8d8fd3701baef2587935b56"
        | "365426f44f8094a10be0113814ce3b2c8bda6d713ef3a120f29589cd37fd5ef8")
}

fn select_owlboy_catalog(root: &Path, target: &Path) -> Result<(), String> {
    const CSV: &str = "Data/Localizations/Translations.csv";
    // The current Steam depot can leave a duplicate final fragment ("ini,-1\r\n").
    // The delta only reads the shared prefix; the journal still backs up the exact input.
    const WITH_TRAILING_FRAGMENT: &str =
        "baa0c87bdb64d3d3c4324f99d9075af088a8b7e39550f46d72e134e1fad874e6";
    let csv = target.join(CSV);
    if !csv.is_file() || super::patch_lifecycle::hash(&csv)? != WITH_TRAILING_FRAGMENT {
        return Ok(());
    }
    let manifest = root.join("patch.json");
    let mut package: super::patch_lifecycle::Package =
        serde_json::from_slice(&fs::read(&manifest).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let change = package.platforms.get_mut("osx")
        .and_then(|plan| plan.files.iter_mut().find(|change| change.path == CSV))
        .ok_or("Missing Owlboy translation file in patch manifest")?;
    change.original_sha256 = Some(WITH_TRAILING_FRAGMENT.into());
    fs::write(manifest, serde_json::to_vec(&package).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
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
