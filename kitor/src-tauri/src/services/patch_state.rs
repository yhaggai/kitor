//! One patch-state decision for the library and launch-profile selection.
use super::{game_behavior::GameBehavior, game_refresh_service::GameHealth};
use crate::models::{game, installed_patch};
use std::path::Path;

pub struct PatchState {
    pub recorded: Vec<installed_patch::Model>,
    pub manual_health: Option<GameHealth>,
}
impl PatchState {
    pub fn is_patched(&self) -> bool {
        self.recorded.iter().any(|patch| patch.status == "active")
            || self
                .manual_health
                .as_ref()
                .is_some_and(|h| h.patch_status == "installed")
    }
}
pub async fn resolve(
    db: &sea_orm::DatabaseConnection,
    game: &game::Model,
    app_data: &Path,
    behavior: &GameBehavior,
) -> Result<PatchState, String> {
    let recorded = super::patch_service::reconcile_patches(db, game, behavior).await?;
    let manual_health = if recorded.is_empty() {
        super::game_refresh_service::inspect_manual(
            behavior,
            app_data,
            Path::new(&game.install_path),
            &game.scummvm_id,
        )
    } else {
        None
    };
    Ok(PatchState {
        recorded,
        manual_health,
    })
}

#[cfg(test)]
mod tests;
