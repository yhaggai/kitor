//! Authenticate the complete archive against the trusted application catalog.
use std::path::Path;

pub fn required_checksum(checksum: Option<&str>) -> Result<&str, String> {
    checksum.filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| "This patch has no trusted SHA-256 checksum. Update Kitor before installing it.".into())
}

pub fn verify(path: &Path, checksum: &str) -> Result<(), String> {
    required_checksum(Some(checksum))?;
    if !super::patch_lifecycle::hash(path)?.eq_ignore_ascii_case(checksum) {
        return Err("Patch archive does not match the trusted catalog checksum; installation refused.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_missing_malformed_and_replaced_archives() {
        assert!(required_checksum(None).is_err());
        assert!(required_checksum(Some("abc")).is_err());
        assert!(required_checksum(Some(&"z".repeat(64))).is_err());
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"reviewed archive").unwrap();
        let trusted = super::super::patch_lifecycle::hash(file.path()).unwrap();
        assert!(verify(file.path(), &trusted.to_uppercase()).is_ok());
        std::fs::write(file.path(), b"replaced archive including its checksums").unwrap();
        assert!(verify(file.path(), &trusted).is_err());
    }
}
