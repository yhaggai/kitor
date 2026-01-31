use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "gog_auth")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
