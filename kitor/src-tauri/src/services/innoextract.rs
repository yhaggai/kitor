use std::path::PathBuf;

pub(super) fn bundled_path() -> Result<PathBuf, String> {
    if !cfg!(any(target_os = "macos", target_os = "windows")) {
        return Err("GOG installer extraction is supported on macOS and Windows.".into());
    }
    let filename = if cfg!(target_os = "windows") {
        "innoextract.exe"
    } else {
        "innoextract"
    };
    let path = std::env::current_exe()
        .map_err(|error| error.to_string())?
        .with_file_name(filename);
    if path.is_file() {
        Ok(path)
    } else {
        Err("Kitor's bundled GOG extraction tool is missing. Reinstall Kitor and retry.".into())
    }
}
