// SPDX-License-Identifier: GPL-2.0-or-later
//! Drizzle-generated schema, native transactions. Blocking entry points are called only on
//! Tauri blocking workers, setup, or synchronous tests, never on async executor workers.
use crate::saves;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Connection, SqliteConnection};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub type Result<T> = std::result::Result<T, String>;
pub fn path(data: &Path) -> PathBuf {
    data.join("launcher.db")
}
pub fn block<T>(future: impl std::future::Future<Output = Result<T>>) -> Result<T> {
    tauri::async_runtime::block_on(future)
}
pub async fn connect(data: &Path) -> Result<SqliteConnection> {
    saves::ensure_directory(data)?;
    let file = path(data);
    if file
        .symlink_metadata()
        .is_ok_and(|m| !m.is_file() || m.file_type().is_symlink())
    {
        return Err("Invalid launcher database file.".into());
    }
    let options = SqliteConnectOptions::new()
        .filename(file)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(10));
    let mut connection = SqliteConnection::connect_with(&options)
        .await
        .map_err(err)?;
    migrate(&mut connection).await?;
    Ok(connection)
}
pub fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}
const MIGRATIONS: &[(&str, &str)] = &[(
    "0000_launcher_storage",
    include_str!("../../src/db/migrations/0000_launcher_storage.sql"),
)];
async fn migrate(connection: &mut SqliteConnection) -> Result<()> {
    sqlx::query("CREATE TABLE IF NOT EXISTS __launcher_migrations (tag TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)")
        .execute(&mut *connection).await.map_err(err)?;
    apply_migrations(connection, MIGRATIONS).await
}
async fn apply_migrations(
    connection: &mut SqliteConnection,
    migrations: &[(&str, &str)],
) -> Result<()> {
    // BEGIN IMMEDIATE serializes first-launch migration by multiple launcher instances.
    for (tag, sql) in migrations {
        let mut tx = connection
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(err)?;
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM __launcher_migrations WHERE tag = ?")
                .bind(tag)
                .fetch_one(&mut *tx)
                .await
                .map_err(err)?;
        if exists == 0 {
            for statement in sql
                .split("--> statement-breakpoint")
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                sqlx::query(statement)
                    .execute(&mut *tx)
                    .await
                    .map_err(err)?;
            }
            sqlx::query("INSERT INTO __launcher_migrations VALUES (?, ?)")
                .bind(tag)
                .bind(now())
                .execute(&mut *tx)
                .await
                .map_err(err)?;
        }
        tx.commit().await.map_err(err)?;
    }
    Ok(())
}
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|t| t.as_secs() as i64)
        .unwrap_or_default()
}
pub fn operation_id() -> String {
    format!("op-{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_migration_rolls_back_schema_and_history() {
        let root = std::fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(operation_id());
        block(async {
            let mut connection = connect(&root).await?;
            let failure = apply_migrations(&mut connection, &[("synthetic_failure", "CREATE TABLE synthetic_atomic (id INTEGER); --> statement-breakpoint THIS IS NOT SQL")]).await;
            assert!(failure.is_err());
            let tables: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlite_master WHERE name='synthetic_atomic'").fetch_one(&mut connection).await.map_err(err)?;
            let history: i64 = sqlx::query_scalar("SELECT count(*) FROM __launcher_migrations WHERE tag='synthetic_failure'").fetch_one(&mut connection).await.map_err(err)?;
            assert_eq!((tables, history), (0,0));
            Ok(())
        }).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn generated_migrations_list_is_complete_and_foreign_keys_are_enforced() {
        let journal: serde_json::Value =
            serde_json::from_str(include_str!("../../src/db/migrations/meta/_journal.json"))
                .unwrap();
        assert_eq!(
            journal["entries"].as_array().unwrap().len(),
            MIGRATIONS.len()
        );
        let root = std::fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(operation_id());
        block(async {
            let mut db = connect(&root).await?;
            assert!(
                sqlx::query("INSERT INTO save_sets VALUES ('orphan','missing','/synthetic')")
                    .execute(&mut db)
                    .await
                    .is_err()
            );
            drop(db);
            let mut db = connect(&root).await?;
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM __launcher_migrations")
                .fetch_one(&mut db)
                .await
                .map_err(err)?;
            assert_eq!(count, MIGRATIONS.len() as i64);
            Ok(())
        })
        .unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
