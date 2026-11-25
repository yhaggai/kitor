//! ScummVM preparation and compatibility exports for shared runtime capabilities.
use super::game_behavior::LaunchProfile;
use std::path::{Path, PathBuf};

pub use super::game_detection::detect;
pub use super::native_launch::native_command;

// Some GOG Book One bundles also contain FATE.PAK. Together with the Hebrew
// font it wins ScummVM's detection as Book Two. Detect using only Book One's
// signature, then load all remaining assets through --extrapath.
pub(super) fn prepare_kyrandia_launch_path(
    data_path: &Path,
    launch_path: &Path,
) -> Result<PathBuf, String> {
    let signature = std::fs::read_dir(data_path)
        .map_err(|e| format!("Failed to read Kyrandia data: {}", e))?
        .flatten()
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("GEMCUT.PAK")
                && entry.path().is_file()
        })
        .ok_or("Kyrandia GEMCUT.PAK not found")?;
    std::fs::create_dir_all(launch_path)
        .map_err(|e| format!("Failed to create Kyrandia launch folder: {}", e))?;
    // Refresh after installing or restoring a patch; the signature changes.
    std::fs::copy(signature.path(), launch_path.join("GEMCUT.PAK"))
        .map_err(|e| format!("Failed to prepare Kyrandia launch: {}", e))?;
    Ok(launch_path.to_path_buf())
}

pub fn scummvm_paths(
    profile: &LaunchProfile,
    root: &Path,
    staging: &Path,
) -> Result<(PathBuf, Option<PathBuf>), String> {
    use super::game_behavior::LaunchPreparation;
    let use_root = matches!(
        profile.preparation,
        Some(LaunchPreparation::TentacleClassic)
    ) && super::games::tentacle::detect_classic(root).is_some();
    let data = if use_root {
        root.to_path_buf()
    } else {
        profile
            .subdirectory
            .as_ref()
            .map(|p| super::patch_lifecycle::safe_path(root, p))
            .transpose()?
            .unwrap_or(root.to_path_buf())
    };
    match profile.preparation {
        Some(LaunchPreparation::KyrandiaSignature) => {
            Ok((prepare_kyrandia_launch_path(&data, staging)?, Some(data)))
        }
        _ => Ok((data, None)),
    }
}

#[cfg(test)]
mod jenny_leclue_tests {
    use super::*;

    #[test]
    fn resolves_steam_and_manual_roots_only_with_matching_game_data() {
        let registry = super::super::registry_service::load_registry().unwrap();
        let game = registry.games.iter().find(|g| g.scummvm_id == "jenny-leclue").unwrap();
        let library = tempfile::tempdir().unwrap();
        let steam = library.path();
        let app = steam.join("Jenny LeClue - Detectivu.app");
        for (root, markers) in [
            (app.as_path(), ["Contents/MacOS/Jenny LeClue - Detectivu", "Contents/Resources/Data/data.unity3d", "Contents/Resources/Data/Managed/DialogueSystem.dll"]),
            (steam, ["Jenny LeClue.exe", "Jenny LeClue_Data/resources.assets", "Jenny LeClue_Data/Managed/DialogueSystem.dll"]),
        ] {
            for (index, marker) in markers.iter().enumerate() {
                let path = root.join(marker);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, []).unwrap();
                if index < 2 {
                    assert_ne!(detect(game, root), root.canonicalize().ok());
                }
            }
            assert_eq!(detect(game, root), root.canonicalize().ok());
            if root == app {
                for _ in 0..2 {
                    assert_eq!(detect(game, steam), app.canonicalize().ok());
                }
            }
        }
    }
}
