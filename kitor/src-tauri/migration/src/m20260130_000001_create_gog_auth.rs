use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GogAuth::Table)
                    .if_not_exists()
                    .col(pk_auto(GogAuth::Id))
                    .col(string(GogAuth::AccessToken))
                    .col(string(GogAuth::RefreshToken))
                    .col(timestamp(GogAuth::ExpiresAt))
                    .col(timestamp(GogAuth::CreatedAt))
                    .col(timestamp(GogAuth::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GogAuth::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GogAuth {
    Table,
    Id,
    AccessToken,
    RefreshToken,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}
