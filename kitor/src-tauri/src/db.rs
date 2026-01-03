use sea_orm::{Database, DatabaseConnection, DbErr};
use migration::{Migrator, MigratorTrait};
use std::path::PathBuf;

pub async fn init_db(app_data_dir: &PathBuf) -> Result<DatabaseConnection, DbErr> {
    // Create app data directory if it doesn't exist
    if !app_data_dir.exists() {
        std::fs::create_dir_all(app_data_dir)
            .map_err(|e| DbErr::Custom(format!("Failed to create app data dir: {}", e)))?;
    }

    let db_path = app_data_dir.join("kitor.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    eprintln!("📁 Database path: {}", db_path.display());

    let db = Database::connect(&db_url).await?;

    // Run migrations
    eprintln!("🔄 Running migrations...");
    Migrator::up(&db, None).await?;
    eprintln!("✅ Migrations complete");

    Ok(db)
}



