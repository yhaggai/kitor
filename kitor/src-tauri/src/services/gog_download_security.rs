use std::path::Path;
use url::Url;

pub fn installer_url(value: &str) -> Result<Url, String> {
    let url = Url::parse(value).map_err(|_| "Invalid GOG installer URL")?;
    if !safe_transport(&url) || url.host_str() != Some("www.gog.com")
        || !url.path().starts_with("/downloads/") || url.query().is_some() {
        return Err("Installer must be a GOG downloads URL".into());
    }
    Ok(url)
}

fn safe_transport(url: &Url) -> bool {
    url.scheme() == "https" && url.username().is_empty() && url.password().is_none()
        && url.port_or_known_default() == Some(443) && url.fragment().is_none()
}

pub fn allowed_redirect(url: &Url) -> bool {
    let host = url.host_str().unwrap_or_default();
    safe_transport(url) && (host == "gog.com" || host.ends_with(".gog.com")
        || host == "gog-statics.com" || host.ends_with(".gog-statics.com"))
        && !url.query_pairs().any(|(name, _)| name.eq_ignore_ascii_case("access_token"))
}

pub fn extraction_directory(parent: &Path) -> Result<tempfile::TempDir, String> {
    if !parent.is_absolute() || !parent.is_dir() {
        return Err("Choose an existing absolute installation directory".into());
    }
    let parent = parent.canonicalize().map_err(|e| e.to_string())?;
    // Never extract over a game, saves, or arbitrary existing files. Keep this
    // guard until extraction succeeds so failures clean up only our new folder.
    tempfile::Builder::new().prefix("kitor-gog-").tempdir_in(parent).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credentials_cannot_be_sent_to_untrusted_urls() {
        assert!(installer_url("https://www.gog.com/downloads/game/en1installer0").is_ok());
        for bad in ["http://www.gog.com/downloads/game", "https://www.gog.com.evil.test/downloads/game",
            "https://evil.test/downloads/game", "https://www.gog.com@evil.test/downloads/game",
            "https://www.gog.com:444/downloads/game", "https://www.gog.com/downloads/game?access_token=x",
            "https://www.gog.com/account"] {
            assert!(installer_url(bad).is_err(), "{bad}");
        }
        for bad in ["http://cdn.gog.com/file", "https://gog-statics.com.evil.test/file",
            "https://evil.test/file", "https://cdn.gog.com/file?access_token=secret"] {
            assert!(!allowed_redirect(&Url::parse(bad).unwrap()), "{bad}");
        }
        assert!(allowed_redirect(&Url::parse("https://cdn.gog.com/file?signature=abc").unwrap()));
    }
    #[test]
    fn extraction_never_overwrites_selected_directory_contents() {
        let parent = tempfile::tempdir().unwrap();
        let original = parent.path().join("save.dat");
        std::fs::write(&original, b"keep").unwrap();
        let extraction = extraction_directory(parent.path()).unwrap();
        std::fs::write(extraction.path().join("save.dat"), b"installer").unwrap();
        let path = extraction.path().to_owned();
        drop(extraction);
        assert!(!path.exists());
        assert_eq!(std::fs::read(original).unwrap(), b"keep");
    }
}
