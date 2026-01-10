use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create games table
        manager
            .create_table(
                Table::create()
                    .table(Games::Table)
                    .if_not_exists()
                    .col(pk_auto(Games::Id))
                    .col(string_uniq(Games::ScummvmId))
                    .col(string(Games::Title))
                    .col(string(Games::InstallPath))
                    .col(string_null(Games::CoverUrl))
                    .col(string_null(Games::DetectedVersion))
                    .col(timestamp(Games::AddedDate))
                    .col(timestamp_null(Games::LastPlayed))
                    .to_owned(),
            )
            .await?;

        // Create installed_patches table
        manager
            .create_table(
                Table::create()
                    .table(InstalledPatches::Table)
                    .if_not_exists()
                    .col(pk_auto(InstalledPatches::Id))
                    .col(integer(InstalledPatches::GameId))
                    .col(string(InstalledPatches::ScummvmId))
                    .col(string(InstalledPatches::Version))
                    .col(timestamp(InstalledPatches::InstalledDate))
                    .col(string(InstalledPatches::Status))
                    .col(string_null(InstalledPatches::BackupPath))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InstalledPatches::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Games::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Games {
    Table,
    Id,
    ScummvmId,
    Title,
    InstallPath,
    CoverUrl,
    DetectedVersion,
    AddedDate,
    LastPlayed,
}

#[derive(DeriveIden)]
enum InstalledPatches {
    Table,
    Id,
    GameId,
    ScummvmId,
    Version,
    InstalledDate,
    Status,
    BackupPath,
}


