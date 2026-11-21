use super::*;
use sea_orm::{ConnectionTrait, Database, DbBackend, EntityTrait, Statement};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Timestamp {
    #[serde(with = "super::serde_naive")]
    value: PrimitiveDateTime,
    #[serde(default, with = "super::serde_naive::option")]
    optional: Option<PrimitiveDateTime>,
}

#[test]
fn frontend_timestamps_preserve_iso_strings_and_nulls() {
    for stamp in [
        "2026-11-21T13:18:05",
        "2026-11-21T13:18:05.120",
        "2026-11-21T13:18:05.123400",
        "2026-11-21T13:18:05.123456789",
    ] {
        for optional in ["null".to_string(), format!("\"{stamp}\"")] {
            let json = format!(r#"{{"value":"{stamp}","optional":{optional}}}"#);
            let parsed: Timestamp = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
        }
    }
}

#[tokio::test]
async fn existing_sqlite_timestamps_remain_readable_and_writable() {
    use crate::models::gog_auth;
    use sea_orm::{ActiveModelTrait, Set};
    let db = Database::connect("sqlite::memory:").await.unwrap();
    db.execute(Statement::from_string(DbBackend::Sqlite,
        "CREATE TABLE gog_auth (id INTEGER PRIMARY KEY, access_token TEXT NOT NULL, refresh_token TEXT NOT NULL, expires_at TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)".to_string()
    )).await.unwrap();
    // Chrono/SQLx writes naive UTC timestamps using a space separator.
    db.execute(Statement::from_string(DbBackend::Sqlite,
        "INSERT INTO gog_auth VALUES (1, 'access', 'refresh', '2026-11-21 13:18:05.123456789', '2026-11-21 13:18:05', '2026-11-21 13:18:05')".to_string()
    )).await.unwrap();
    let auth = gog_auth::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(auth.expires_at.nanosecond(), 123456789);
    assert_eq!(
        auth.created_at,
        time::macros::datetime!(2026-11-21 13:18:05)
    );
    let expected = auth.expires_at + time::Duration::hours(1);
    let mut active: gog_auth::ActiveModel = auth.into();
    active.expires_at = Set(expected);
    active.update(&db).await.unwrap();
    assert_eq!(
        gog_auth::Entity::find_by_id(1)
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .expires_at,
        expected
    );
}

#[test]
fn utc_conversion_preserves_the_instant() {
    let value = time::macros::datetime!(2026-11-21 15:18:05 +02:00);
    assert_eq!(
        naive_utc(value),
        time::macros::datetime!(2026-11-21 13:18:05)
    );
    assert!(OffsetDateTime::parse(&now_rfc3339(), &Rfc3339).is_ok());
}
