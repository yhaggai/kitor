# SeaORM Migration System

## Structure

```
src-tauri/
├── migration/                              # Migration crate
│   ├── Cargo.toml                          # Migration dependencies
│   └── src/
│       ├── lib.rs                          # Migrator registration
│       ├── m20260103_000001_create_tables.rs
│       └── m20260103_000002_add_launch_columns.rs
├── src/
│   └── db.rs                               # Calls Migrator::up()
└── Cargo.toml                              # Depends on migration crate
```

## How It Works

### 1. Migrations Run Automatically
When `init_db()` is called on app startup, it runs:
```rust
Migrator::up(&db, None).await?;
```

### 2. Migration Tracking
SeaORM creates a `seaql_migrations` table to track which migrations have run.

### 3. Idempotent
- Migrations only run once
- Already-run migrations are skipped
- Safe to restart app multiple times

## Current Migrations

### m20260103_000001_create_tables
- Creates `games` table with base columns
- Creates `installed_patches` table
- **Status**: Will run on fresh DB

### m20260103_000002_add_launch_columns
- Adds `launch_subdirectory` column to games
- Adds `launch_args` column to games
- Uses `has_column()` check (MySQL compatible)
- **Status**: Will run on existing DB or after migration 1

## Adding New Migrations

1. Create new file: `migration/src/mYYYYMMDD_HHMMSS_name.rs`
2. Implement `MigrationTrait` with `up()` and `down()`
3. Register in `migration/src/lib.rs`

Example:
```rust
// migration/src/m20260104_000001_add_new_field.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("games", "new_field").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Games::Table)
                        .add_column(ColumnDef::new(Games::NewField).string())
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
                    .drop_column(Games::NewField)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Games {
    Table,
    NewField,
}
```

## Benefits

✅ Versioned schema changes
✅ Rollback support (down migrations)
✅ Multi-backend compatible (SQLite, Postgres, MySQL)
✅ Automatic tracking of applied migrations
✅ No manual SQL needed
✅ Type-safe with SeaQuery


