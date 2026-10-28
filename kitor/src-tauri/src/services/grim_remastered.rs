//! Bundled native Windows Grim Fandango Remastered Hebrew candidate.
use std::{fs, path::Path, process::Command};
pub const VERSION: &str = "0.3.0-native-candidate";
pub fn stage(destination: &Path, language: &str, version: &str) -> Result<(), String> {
    if !cfg!(target_os = "windows") { return Err("Grim Fandango Hebrew supports native Windows only".into()); }
    if language != "he" || version != VERSION { return Err("Unknown Grim Fandango patch version".into()); }
    for (name, bytes) in [
        ("patch.json", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/patch.windows.candidate.json").as_slice()),
        ("payload/FontsHD/COMICSANS18.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/COMICSANS18.LAF.txt").as_slice()),
        ("payload/FontsHD/FiraSans-Regular_0.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/FiraSans-Regular_0.ttf").as_slice()),
        ("payload/FontsHD/FUTURA40.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/FUTURA40.LAF.txt").as_slice()),
        ("payload/FontsHD/FuturaStd-Heavy.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/FuturaStd-Heavy.ttf").as_slice()),
        ("payload/FontsHD/FuturaStd-Medium.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/FuturaStd-Medium.ttf").as_slice()),
        ("payload/FontsHD/HebrewPrototype.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/HebrewPrototype.ttf").as_slice()),
        ("payload/FontsHD/KINO14S.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/KINO14S.LAF.txt").as_slice()),
        ("payload/FontsHD/Koloss.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/Koloss.ttf").as_slice()),
        ("payload/FontsHD/KOLOSS100.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/KOLOSS100.LAF.txt").as_slice()),
        ("payload/FontsHD/KOLOSS52.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/KOLOSS52.LAF.txt").as_slice()),
        ("payload/FontsHD/KOLOSS69.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/KOLOSS69.LAF.txt").as_slice()),
        ("payload/FontsHD/Meredith.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/Meredith.ttf").as_slice()),
        ("payload/FontsHD/MEREDITH38.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/MEREDITH38.LAF.txt").as_slice()),
        ("payload/FontsHD/MEREDITH46.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/MEREDITH46.LAF.txt").as_slice()),
        ("payload/FontsHD/OCR10.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/OCR10.LAF.txt").as_slice()),
        ("payload/FontsHD/OCRA.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/OCRA.ttf").as_slice()),
        ("payload/FontsHD/OCRB.ttf", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/OCRB.ttf").as_slice()),
        ("payload/FontsHD/TREB13BS.LAF.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/FontsHD/TREB13BS.LAF.txt").as_slice()),
        ("payload/grim.he.tab", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/payload/grim.he.tab").as_slice()),
        ("licenses/CREDITS.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/licenses/CREDITS.txt").as_slice()),
        ("licenses/FONT-OFL.txt", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/licenses/FONT-OFL.txt").as_slice()),
        ("recipes/GrimFandango.exe.json", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/recipes/GrimFandango.exe.json").as_slice()),
        ("recipes/DATA000.LAB.json", include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/recipes/DATA000.LAB.json").as_slice()),
    ] {
        let path = destination.join(name);
        fs::create_dir_all(path.parent().ok_or("Invalid payload path")?).map_err(|e| e.to_string())?;
        fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}
pub fn launch_command(root: &Path) -> Result<Command, String> {
    if !cfg!(target_os = "windows") { return Err("Grim Fandango native launch supports Windows only".into()); }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    if !root.join("GrimFandango.exe").is_file() || !root.join("grim.en.tab").is_file() {
        return Err("Native Grim Fandango Remastered installation not found".into());
    }
    let mut command = Command::new(root.join("GrimFandango.exe"));
    command.current_dir(root);
    Ok(command)
}

// The native game has no language-index command-line switch. Its version-2
// registry stores 64 records (Adler-32 key, type, 260-byte value), then 24 bytes
// of other settings. Change only Language, a type-3 float, before starting it.
fn hebrew_preferences(original: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut data = original.map(Vec::from).unwrap_or_else(|| {
        let mut bytes = vec![0; 0x431c];
        bytes[..4].copy_from_slice(&2u32.to_le_bytes());
        bytes
    });
    if data.len() != 0x431c || data[..4] != 2u32.to_le_bytes() {
        return Err("Unsupported Grim Fandango settings format; settings were not changed".into());
    }
    let key = 0x0dd00325u32.to_le_bytes(); // Adler-32("Language")
    let slots: Vec<_> = (0..64).map(|i| 4 + i * 268).collect();
    let slot = slots.iter().copied().find(|&i| data[i..i + 4] == key)
        .or_else(|| slots.iter().copied().find(|&i| data[i..i + 268].iter().all(|&b| b == 0)))
        .ok_or("Grim Fandango settings have no free language record")?;
    if data[slot..slot + 4] == key && data[slot + 4..slot + 8] != 3u32.to_le_bytes() {
        return Err("Unsupported Grim Fandango language setting type".into());
    }
    data[slot..slot + 4].copy_from_slice(&key);
    data[slot + 4..slot + 8].copy_from_slice(&3u32.to_le_bytes());
    data[slot + 8..slot + 12].copy_from_slice(&6f32.to_le_bytes());
    Ok(data)
}

pub fn prepare_launch(root: &Path) -> Result<(), String> {
    if !root.join("grim.he.tab").is_file() { return Ok(()); }
    let appdata = std::env::var_os("APPDATA").ok_or("APPDATA is unavailable")?;
    let directory = Path::new(&appdata).join("Doublefine/GrimFandango");
    let path = directory.join("registry.sav");
    let original = match fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
    };
    let updated = hebrew_preferences(original.as_deref())?;
    if original.as_deref() == Some(updated.as_slice()) { return Ok(()); }
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let backup = directory.join("registry.sav.kitor-before-he");
    if original.is_some() && !backup.exists() {
        fs::copy(&path, backup).map_err(|e| e.to_string())?;
    }
    use std::io::Write;
    let mut staged = tempfile::NamedTempFile::new_in(&directory).map_err(|e| e.to_string())?;
    staged.write_all(&updated).map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    staged.persist(&path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_hebrew_without_changing_other_preferences() {
        let mut original = hebrew_preferences(None).unwrap();
        original[16..20].copy_from_slice(b"keep");
        original[0x4304..].fill(42);
        original[12..16].copy_from_slice(&0f32.to_le_bytes());
        let updated = hebrew_preferences(Some(&original)).unwrap();
        assert_eq!(&updated[..12], &original[..12]);
        assert_eq!(&updated[16..], &original[16..]);
        assert_eq!(&updated[12..16], &6f32.to_le_bytes());
        assert_eq!(hebrew_preferences(Some(&updated)).unwrap(), updated);
    }
    #[test]
    fn rejects_unknown_settings_without_replacing_them() {
        assert!(hebrew_preferences(Some(b"unknown")).is_err());
    }
}
