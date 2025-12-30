use sea_orm::{Database, DatabaseConnection, DbErr};
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
    create_tables(&db).await?;

    Ok(db)
}

async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::sea_query::{ColumnDef, SqliteQueryBuilder, Table};
    use sea_orm::{ConnectionTrait, Statement};

    // Drop installed_patches table to recreate without language column
    let drop_patches = "DROP TABLE IF EXISTS installed_patches";
    db.execute(Statement::from_string(db.get_database_backend(), drop_patches))
        .await?;

    // Create games table
    let create_games = Table::create()
        .table(crate::models::game::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(crate::models::game::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(crate::models::game::Column::ScummvmId)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(crate::models::game::Column::Title)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(crate::models::game::Column::InstallPath)
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(crate::models::game::Column::CoverUrl).string())
        .col(ColumnDef::new(crate::models::game::Column::DetectedVersion).string())
        .col(
            ColumnDef::new(crate::models::game::Column::AddedDate)
                .timestamp()
                .not_null(),
        )
        .col(ColumnDef::new(crate::models::game::Column::LastPlayed).timestamp())
        .build(SqliteQueryBuilder);

    db.execute(Statement::from_string(db.get_database_backend(), create_games))
        .await?;

    // Create installed_patches table
    let create_installed = Table::create()
        .table(crate::models::installed_patch::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(crate::models::installed_patch::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(crate::models::installed_patch::Column::GameId)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(crate::models::installed_patch::Column::ScummvmId)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(crate::models::installed_patch::Column::Version)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(crate::models::installed_patch::Column::InstalledDate)
                .timestamp()
                .not_null(),
        )
        .col(
            ColumnDef::new(crate::models::installed_patch::Column::Status)
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(crate::models::installed_patch::Column::BackupPath).string())
        .build(SqliteQueryBuilder);

    db.execute(Statement::from_string(db.get_database_backend(), create_installed))
        .await?;

    Ok(())
}



