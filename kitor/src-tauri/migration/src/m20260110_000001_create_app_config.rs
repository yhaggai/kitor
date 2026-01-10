use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppConfig::Table)
                    .if_not_exists()
                    .col(string(AppConfig::Key).primary_key())
                    .col(string(AppConfig::Value))
                    .col(timestamp(AppConfig::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AppConfig {
    Table,
    Key,
    Value,
    UpdatedAt,
}
