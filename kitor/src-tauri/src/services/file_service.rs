use std::path::Path;
use std::fs;
use std::io;

/// Read every entry so truncated archives and CRC failures are caught before
/// installation creates a backup or writes any game files.
pub fn validate_zip(path: &Path) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|e| format!("Failed to open downloaded ZIP: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Downloaded file is not a valid ZIP archive: {}", e))?;
    if archive.is_empty() {
        return Err("Downloaded ZIP archive is empty".to_string());
    }
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| format!("Invalid ZIP entry: {}", e))?;
        if entry.enclosed_name().is_none() {
            return Err("Downloaded ZIP contains an unsafe file path".to_string());
        }
        io::copy(&mut entry, &mut io::sink())
            .map_err(|e| format!("Downloaded ZIP is incomplete or corrupt: {}", e))?;
    }
    Ok(())
}

pub fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open zip file: {}", e))?;
    
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read file from archive: {}", e))?;
        
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;
            io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }

    Ok(())
}

