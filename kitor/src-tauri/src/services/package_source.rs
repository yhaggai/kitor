//! Stage verified translation packages before touching game files.
use super::{game_behavior::GameBehavior, registry_service::PatchDefinition};
use std::{fs, path::Path};
include!(concat!(env!("OUT_DIR"), "/bundled_assets.rs"));

pub enum PackageSource {
    Download,
    #[cfg(debug_assertions)]
    LocalPreview(std::path::PathBuf),
    OwlboyMacSteam,
    SpaceQuest4Scummvm,
    Embedded(String),
}
impl PackageSource {
    pub fn resolve(patch: &PatchDefinition, behavior: &GameBehavior) -> Result<Self, String> {
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
            if let (Some(key), None) = (&behavior.bundled_resource, behavior.bundled) {
                return Ok(Self::Embedded(key.clone()));
            }
            return Err("Bundled patch has no reviewed embedded resource".into());
        }
        #[cfg(debug_assertions)]
        if patch.source_type == "local-preview" && patch.download_url == "local-preview://disco-elysium" {
            let path = super::games::disco_elysium::preview_archive()?.ok_or("Local preview is not enabled")?;
            if patch.checksum.as_deref() != Some(super::patch_lifecycle::hash(&path)?.as_str()) {
                return Err("Local preview changed; refresh before installing".into());
            }
            return Ok(Self::LocalPreview(path));
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
        if let Self::Embedded(key) = self {
            let key = format!("{key}/{}", super::patch_lifecycle::platform());
            let assets = embedded_assets(&key)
                .ok_or("Bundled patch is unavailable for this platform")?;
            for (name, bytes) in assets {
                let path = super::patch_lifecycle::safe_path(destination, name)?;
                fs::create_dir_all(path.parent().ok_or("Invalid asset path")?)
                    .map_err(|e| e.to_string())?;
                fs::write(path, bytes).map_err(|e| e.to_string())?;
            }
            return Ok(());
        }
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
