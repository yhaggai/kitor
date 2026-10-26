//! Remote data can update known translations, not game detection, launch
//! commands, or installer scripts. The bundled catalog remains the fallback.
use super::*;
use std::{collections::HashSet, path::PathBuf, sync::OnceLock};

pub fn repository() -> &'static str {
    "yhaggai/kitor"
}
pub fn url() -> String {
    format!(
        "https://raw.githubusercontent.com/{}/patch-registry/registry.json",
        repository()
    )
}
static CACHE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub catalog_revision: u64,
    pub patches: Vec<Release>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub scummvm_id: String,
    pub language: String,
    pub version: String,
    pub release_revision: u64,
    pub download_url: String,
    pub filename: String,
    pub checksum: String,
    pub file_size: u64,
    pub author: String,
    pub description: String,
    pub platforms: Vec<String>,
}

pub fn initialize(app_data: &Path) {
    let _ = CACHE.set(app_data.join("patch-registry.json"));
}
pub fn cache_path() -> Result<&'static Path, String> {
    CACHE
        .get()
        .map(|p| p.as_path())
        .ok_or("Registry cache is not initialized".into())
}
fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        && value != "."
        && value != ".."
}
pub fn parse(bytes: &[u8]) -> Result<Catalog, String> {
    if bytes.len() > 2_000_000 {
        return Err("Patch registry is too large".into());
    }
    let catalog: Catalog = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if catalog.schema_version != 1 || catalog.catalog_revision == 0 {
        return Err("Unsupported patch registry".into());
    }
    let mut identities = HashSet::new();
    for release in &catalog.patches {
        let tag = format!(
            "patch-{}-{}-v{}",
            release.scummvm_id, release.language, release.version
        );
        let expected = format!(
            "https://github.com/{}/releases/download/{tag}/{}",
            repository(),
            release.filename
        );
        if !token(&release.scummvm_id)
            || !token(&release.language)
            || !token(&release.version)
            || !token(&release.filename)
            || !release.filename.ends_with(".zip")
            || release.download_url != expected
            || release.release_revision == 0
            || release.release_revision > 9_007_199_254_740_991
            || release.checksum.len() != 64
            || !release.checksum.bytes().all(|c| c.is_ascii_hexdigit())
            || release.file_size == 0
            || release.file_size > i64::MAX as u64
            || release.platforms.is_empty()
            || release
                .platforms
                .iter()
                .any(|p| !["windows", "osx"].contains(&p.as_str()))
            || !identities.insert((&release.scummvm_id, &release.language))
        {
            return Err(format!(
                "Invalid or duplicate translation release: {}",
                release.scummvm_id
            ));
        }
    }
    Ok(catalog)
}

pub fn merge(registry: &mut Registry, catalog: &Catalog) {
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    };
    for release in &catalog.patches {
        if !release.platforms.iter().any(|p| p == platform) {
            continue;
        }
        let Some(game) = registry
            .games
            .iter_mut()
            .find(|g| g.scummvm_id == release.scummvm_id)
        else {
            continue;
        };
        let Some(patch) = game
            .patches
            .iter_mut()
            .find(|p| p.language == release.language)
        else {
            continue;
        };
        if release.release_revision <= patch.release_revision {
            continue;
        }
        // A release version identifies immutable bytes. Never replace it in-place.
        if release.version == patch.version {
            continue;
        }
        patch.version = release.version.clone();
        patch.release_revision = release.release_revision;
        patch.download_url = release.download_url.clone();
        patch.filename = Some(release.filename.clone());
        patch.checksum = Some(release.checksum.to_lowercase());
        patch.file_size = Some(release.file_size as i64);
        patch.author = release.author.clone();
        patch.description = release.description.clone();
        patch.source_type = "github".into();
        patch.install_manifest = Some("patch.json".into());
        patch.post_install_script = None;
    }
}

pub fn merge_cached(registry: &mut Registry) {
    if let Ok(path) = cache_path() {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(catalog) = parse(&bytes) {
                merge(registry, &catalog);
            }
        }
    }
}

pub fn validate_advance(previous: &Catalog, next: &Catalog) -> Result<(), String> {
    if next.catalog_revision < previous.catalog_revision {
        return Err("Registry revision moved backwards".into());
    }
    if next.catalog_revision == previous.catalog_revision
        && serde_json::to_vec(previous).unwrap() != serde_json::to_vec(next).unwrap()
    {
        return Err("Published registry revision changed in place".into());
    }
    for old in &previous.patches {
        let new = next
            .patches
            .iter()
            .find(|p| p.scummvm_id == old.scummvm_id && p.language == old.language)
            .ok_or("Registry removed a published translation")?;
        if new.release_revision < old.release_revision
            || (new.release_revision == old.release_revision
                && serde_json::to_vec(new).unwrap() != serde_json::to_vec(old).unwrap())
            || (new.release_revision > old.release_revision && new.version == old.version)
        {
            return Err("Published translation revision regressed or changed in place".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn catalog() -> Catalog {
        Catalog {
            schema_version: 1,
            catalog_revision: 1,
            patches: vec![Release {
                scummvm_id: "sq1".into(),
                language: "he".into(),
                version: "RC2-kitor1".into(),
                release_revision: 1,
                download_url: format!(
                    "https://github.com/{}/releases/download/patch-sq1-he-vRC2-kitor1/patch.zip",
                    repository()
                ),
                filename: "patch.zip".into(),
                checksum: "a".repeat(64),
                file_size: 123,
                author: "Translator".into(),
                description: "Corrected dialogue".into(),
                platforms: vec!["windows".into(), "osx".into()],
            }],
        }
    }
    #[test]
    fn rejects_unsafe_or_ambiguous_catalogs() {
        let good = catalog();
        assert!(parse(&serde_json::to_vec(&good).unwrap()).is_ok());
        for case in 0..5 {
            let mut bad = good.clone();
            match case {
                0 => bad.patches[0].download_url = "https://example.com/patch.zip".into(),
                1 => bad.patches[0].filename = "../patch.zip".into(),
                2 => bad.patches[0].release_revision = 0,
                3 => bad.patches.push(bad.patches[0].clone()),
                _ => bad.schema_version = 2,
            }
            assert!(parse(&serde_json::to_vec(&bad).unwrap()).is_err());
        }
    }
    #[test]
    fn rejects_catalog_rollback_and_same_revision_mutation() {
        let old = catalog();
        assert!(validate_advance(&old, &old).is_ok());
        let mut new = old.clone();
        new.catalog_revision = 0;
        assert!(validate_advance(&old, &new).is_err());
        new.catalog_revision = 2;
        new.patches[0].checksum = "b".repeat(64);
        assert!(validate_advance(&old, &new).is_err());
        new.patches[0].release_revision = 2;
        assert!(validate_advance(&old, &new).is_err());
        new.patches[0].version = "RC2-kitor2".into();
        assert!(validate_advance(&old, &new).is_ok());
        new.patches.clear();
        assert!(validate_advance(&old, &new).is_err());
    }
    #[test]
    fn catalog_never_replaces_launch_or_detection_configuration() {
        let mut registry: Registry =
            serde_json::from_str(include_str!("../../../patches/registry.json")).unwrap();
        let before = registry
            .games
            .iter()
            .find(|g| g.scummvm_id == "sq1")
            .unwrap()
            .clone();
        merge(&mut registry, &catalog());
        let after = registry
            .games
            .iter()
            .find(|g| g.scummvm_id == "sq1")
            .unwrap();
        assert_eq!(
            serde_json::to_value(&before.launch_config).unwrap(),
            serde_json::to_value(&after.launch_config).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&before.detection_patterns).unwrap(),
            serde_json::to_value(&after.detection_patterns).unwrap()
        );
        if cfg!(any(target_os = "macos", target_os = "windows")) {
            assert_eq!(after.patches[0].version, "RC2-kitor1");
            assert_eq!(after.patches[0].release_revision, 1);
            assert!(after.patches[0].post_install_script.is_none());
        }
    }
}
