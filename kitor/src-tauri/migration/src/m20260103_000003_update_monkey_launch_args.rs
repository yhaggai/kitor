use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        // Update launch config for monkey-se games
        let new_args = r#"["--language=he","--fullscreen","--auto-detect"]"#;
        let subdirectory = "hebrew";
        
        db.execute_unprepared(&format!(
            "UPDATE games SET launch_subdirectory = '{}', launch_args = '{}' WHERE scummvm_id = 'monkey-se'",
            subdirectory, new_args
        ))
        .await?;
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        // Revert launch config for monkey-se games
        db.execute_unprepared(
            "UPDATE games SET launch_subdirectory = NULL, launch_args = NULL WHERE scummvm_id = 'monkey-se'"
        )
        .await?;
        
        Ok(())
    }
}

