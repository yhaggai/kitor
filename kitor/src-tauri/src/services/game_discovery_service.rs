//! Local storefront discovery. No account credentials or network access required.
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Default, Serialize)]
pub struct ScanResult {
    pub added_count: usize,
    pub locations: Vec<String>,
    pub warnings: Vec<String>,
}

// Valve's quoted KeyValues strings, including escaped Windows paths.
fn quoted_strings(input: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            for c in chars.by_ref() {
                if c == '\n' {
                    break;
                }
            }
        } else if c == '"' {
            let mut value = String::new();
            while let Some(c) = chars.next() {
                match c {
                    '"' => break,
                    '\\' => match chars.peek() {
                        Some('\\' | '"') => value.push(chars.next().unwrap()),
                        _ => value.push(c),
                    },
                    _ => value.push(c),
                }
            }
            strings.push(value);
        }
    }
    strings
}

fn steam_libraries(contents: &str) -> Vec<PathBuf> {
    quoted_strings(contents)
        .windows(2)
        .filter_map(|pair| {
            let key = &pair[0];
            let value = &pair[1];
            // Modern libraryfolders.vdf uses "path"; older versions use numeric keys.
            if key == "path"
                || (!key.is_empty()
                    && key.chars().all(|c| c.is_ascii_digit())
                    && (value.contains('/') || value.contains('\\')))
            {
                Some(PathBuf::from(value))
            } else {
                None
            }
        })
        .collect()
}

fn add_directory(path: PathBuf, locations: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>) {
    if path.is_dir() {
        let canonical = path.canonicalize().unwrap_or(path);
        if seen.insert(canonical.clone()) {
            locations.push(canonical);
        }
    }
}

fn discover(steam_roots: Vec<PathBuf>, gog_roots: Vec<PathBuf>) -> (Vec<PathBuf>, Vec<String>) {
    let mut locations = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();
    for root in steam_roots {
        add_directory(root.join("steamapps/common"), &mut locations, &mut seen);
        for config in [
            root.join("steamapps/libraryfolders.vdf"),
            root.join("config/libraryfolders.vdf"),
        ] {
            match std::fs::read_to_string(&config) {
                Ok(contents) => {
                    for library in steam_libraries(&contents) {
                        add_directory(library.join("steamapps/common"), &mut locations, &mut seen);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => warnings.push(format!("Could not read {}: {}", config.display(), e)),
            }
        }
    }
    for root in gog_roots {
        add_directory(root, &mut locations, &mut seen);
    }
    (locations, warnings)
}

#[cfg(target_os = "windows")]
fn registry_paths(key: &str, value: &str, recursive: bool) -> Vec<PathBuf> {
    let mut command = std::process::Command::new("reg");
    command.args(["query", key, "/v", value]);
    if recursive {
        command.arg("/s");
    }
    command
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| {
                    line.split_once("REG_SZ")
                        .map(|(_, path)| PathBuf::from(path.trim()))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn discover_locations() -> (Vec<PathBuf>, Vec<String>) {
    let mut steam = Vec::new();
    let mut gog = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        #[cfg(target_os = "macos")]
        {
            steam.push(home.join("Library/Application Support/Steam"));
            gog.extend([
                home.join("GOG Games"),
                home.join("Applications"),
                PathBuf::from("/Applications"),
            ]);
        }
        #[cfg(target_os = "linux")]
        {
            steam.extend([
                home.join(".steam/steam"),
                home.join(".local/share/Steam"),
                home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
                home.join("snap/steam/common/.local/share/Steam"),
            ]);
            gog.extend([home.join("GOG Games"), home.join("Games")]);
        }
    }
    #[cfg(target_os = "windows")]
    {
        steam.extend(registry_paths(
            r"HKCU\Software\Valve\Steam",
            "SteamPath",
            false,
        ));
        for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(base) = std::env::var_os(variable) {
                let base = PathBuf::from(base);
                steam.push(base.join("Steam"));
                gog.push(base.join("GOG Galaxy/Games"));
            }
        }
        for key in [
            r"HKLM\SOFTWARE\GOG.com\Games",
            r"HKLM\SOFTWARE\WOW6432Node\GOG.com\Games",
        ] {
            gog.extend(registry_paths(key, "path", true));
        }
        if let Some(drive) = std::env::var_os("SystemDrive") {
            gog.push(PathBuf::from(format!(
                "{}\\GOG Games",
                drive.to_string_lossy()
            )));
        }
    }
    discover(steam, gog)
}

pub async fn scan_installed_games(
    db: &sea_orm::DatabaseConnection,
    app_data_dir: PathBuf,
) -> Result<ScanResult, String> {
    let (locations, warnings) = tokio::task::spawn_blocking(discover_locations)
        .await
        .map_err(|e| e.to_string())?;
    let mut result = ScanResult {
        warnings,
        ..Default::default()
    };
    for location in locations {
        let path = location.to_string_lossy().into_owned();
        result.locations.push(path.clone());
        match super::game_service::scan_directory(db, &path, app_data_dir.clone()).await {
            Ok(games) => result.added_count += games.len(),
            Err(error) => result.warnings.push(format!("{}: {}", path, error)),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_modern_and_legacy_steam_paths() {
        let paths = steam_libraries(
            r#""libraryfolders" { "0" { "path" "C:\\Steam" "apps" { "123" "456" } } "1" "/mnt/Games" } // "path" "/ignored""#,
        );
        assert_eq!(
            paths,
            vec![PathBuf::from(r"C:\Steam"), PathBuf::from("/mnt/Games")]
        );
    }

    #[test]
    fn discovers_custom_libraries_and_deduplicates_existing_folders() {
        let temp = tempfile::tempdir().unwrap();
        let steam = temp.path().join("Steam");
        let custom = temp.path().join("Custom Library");
        let gog = temp.path().join("GOG Games");
        for path in [&steam, &custom] {
            std::fs::create_dir_all(path.join("steamapps/common")).unwrap();
        }
        std::fs::create_dir_all(&gog).unwrap();
        std::fs::write(
            steam.join("steamapps/libraryfolders.vdf"),
            format!("\"path\" \"{}\"", custom.display()),
        )
        .unwrap();
        let (locations, warnings) = discover(
            vec![steam.clone(), steam],
            vec![gog.clone(), gog, temp.path().join("missing")],
        );
        assert_eq!(locations.len(), 3);
        assert!(locations.contains(&custom.join("steamapps/common").canonicalize().unwrap()));
        assert!(warnings.is_empty());
    }
}
