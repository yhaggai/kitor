pub use sea_orm_migration::prelude::*;

mod m20260103_000001_create_tables;
mod m20260103_000002_add_launch_columns;
mod m20260103_000003_update_monkey_launch_args;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260103_000001_create_tables::Migration),
            Box::new(m20260103_000002_add_launch_columns::Migration),
            Box::new(m20260103_000003_update_monkey_launch_args::Migration),
        ]
    }
}

