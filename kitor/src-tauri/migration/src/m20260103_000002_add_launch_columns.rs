use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add launch_subdirectory column if it doesn't exist
        if !manager.has_column("games", "launch_subdirectory").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Games::Table)
                        .add_column(ColumnDef::new(Games::LaunchSubdirectory).string())
                        .to_owned(),
                )
                .await?;
        }

        // Add launch_args column if it doesn't exist
        if !manager.has_column("games", "launch_args").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Games::Table)
                        .add_column(ColumnDef::new(Games::LaunchArgs).string())
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Games::Table)
                    .drop_column(Games::LaunchSubdirectory)
                    .drop_column(Games::LaunchArgs)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Games {
    Table,
    LaunchSubdirectory,
    LaunchArgs,
}


