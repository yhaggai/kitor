//! Refuse recursive runtime cleanup across filesystem or root-symlink boundaries.
use std::{fs, path::Path};

#[cfg(unix)]
fn same_filesystem_tree(path: &Path, device: u64) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    // remove_dir_all removes nested links themselves, never their targets.
    if metadata.file_type().is_symlink() { return Ok(()); }
    if metadata.dev() != device {
        return Err(format!("Refusing cleanup across a mounted filesystem: {}. Eject it and retry.", path.display()));
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            same_filesystem_tree(&entry.map_err(|e| e.to_string())?.path(), device)?;
        }
    }
    Ok(())
}

pub fn remove_owned_tree(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Runtime cleanup requires a real directory, not a link".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let parent = path.parent().ok_or("Cannot clean a filesystem root")?;
        let device = fs::metadata(parent).map_err(|e| e.to_string())?.dev();
        // Includes the root itself, so a volume mounted at scummvm is rejected.
        same_filesystem_tree(path, device)?;
    }
    fs::remove_dir_all(path).map_err(|e| format!("Runtime cleanup failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleanup_preserves_sibling_files() {
        let dir = tempfile::tempdir().unwrap();
        let owned = dir.path().join("scummvm");
        let sibling = dir.path().join("scummvm-other");
        fs::create_dir(&owned).unwrap();
        fs::create_dir(&sibling).unwrap();
        fs::write(sibling.join("save"), b"keep").unwrap();
        remove_owned_tree(&owned).unwrap();
        assert_eq!(fs::read(sibling.join("save")).unwrap(), b"keep");
    }
    #[cfg(unix)]
    #[test]
    fn cleanup_rejects_root_links_and_device_changes() {
        use std::os::unix::{fs::symlink, fs::MetadataExt};
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        let link = dir.path().join("scummvm");
        fs::create_dir(&real).unwrap();
        fs::write(real.join("save"), b"keep").unwrap();
        symlink(&real, &link).unwrap();
        assert!(remove_owned_tree(&link).is_err());
        let device = fs::metadata(&real).unwrap().dev();
        assert!(same_filesystem_tree(&real, device.wrapping_add(1)).is_err());
        assert_eq!(fs::read(real.join("save")).unwrap(), b"keep");
        let owned = dir.path().join("owned");
        fs::create_dir(&owned).unwrap();
        symlink(&real, owned.join("nested-link")).unwrap();
        remove_owned_tree(&owned).unwrap();
        assert_eq!(fs::read(real.join("save")).unwrap(), b"keep");
    }
}
