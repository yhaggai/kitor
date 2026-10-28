//! All sources produce the same staged manifest and payload layout.
use super::{
    game_behavior::{BundledAdapter, GameBehavior},
    registry_service::PatchDefinition,
};
use std::path::Path;
include!(concat!(env!("OUT_DIR"), "/bundled_assets.rs"));

pub enum PackageSource<'a> {
    Download,
    Embedded(&'a str),
    Adapter(BundledAdapter),
}
impl<'a> PackageSource<'a> {
    pub fn resolve(patch: &PatchDefinition, behavior: &'a GameBehavior) -> Result<Self, String> {
        if patch.source_type != "bundled" {
            return Ok(Self::Download);
        }
        match (&behavior.bundled_resource, behavior.bundled) {
            (Some(key), None) => Ok(Self::Embedded(key)),
            (None, Some(adapter)) => Ok(Self::Adapter(adapter)),
            _ => Err("Bundled patch must select exactly one resource or adapter".into()),
        }
    }
    pub fn stage(
        &self,
        destination: &Path,
        archive: &Path,
        id: &str,
        language: &str,
        version: &str,
        target: &Path,
    ) -> Result<(), String> {
        match self {
            Self::Download => {
                super::file_service::validate_zip(archive)?;
                super::file_service::extract_zip(archive, destination)
            }
            Self::Embedded(key) => {
                let key = format!("{key}/{}", super::patch_lifecycle::platform());
                let assets = embedded_assets(&key)
                    .ok_or("Bundled package is unavailable for this platform")?;
                for (name, bytes) in assets {
                    let path = super::patch_lifecycle::safe_path(destination, name)?;
                    std::fs::create_dir_all(path.parent().ok_or("Invalid asset path")?)
                        .map_err(|e| e.to_string())?;
                    std::fs::write(path, bytes).map_err(|e| e.to_string())?;
                }
                Ok(())
            }
            Self::Adapter(adapter) => {
                super::game_adapters::stage(*adapter, destination, id, language, version, target)
            }
        }
    }
}

#[cfg(test)]
mod tests;
