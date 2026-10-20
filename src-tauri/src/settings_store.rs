// SPDX-License-Identifier: GPL-2.0-or-later
//! Settings in the launcher database, shared by all profiles: the launcher's options (scope
//! "launcher", JSON values) and the game's (scope "game", bbport.ini's keys and text values).
//! launcher.json and bbport.ini are imported once, the first time the table is empty, and kept
//! as they were. bbport.ini is written from here when the game starts (run.sh, the patch scripts
//! and the game read it); the in-game menu sends its changes back over the game's output
//! ("BBSET key=value", "BBSET launcher:key=value" for GAME_SYNCED) and receives the launcher's on
//! its input.
use crate::database as db;
use sqlx::{Connection, Row, SqliteConnection};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const LAUNCHER: &str = "launcher";
pub const GAME: &str = "game";

fn launcher_json(data: &Path) -> std::path::PathBuf {
    data.join("launcher.json")
}
pub fn ini_path(data: &Path) -> std::path::PathBuf {
    data.join("bbport.ini")
}

/// bbport.ini's key=value lines (comments and blank lines skipped).
pub fn parse_ini(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

/// The launcher's options the in-game menu also shows and changes ("BBSET launcher:key=value";
/// the game starts with them in its environment).
pub const GAME_SYNCED: [&str; 10] = [
    "fullscreen", "display", "present_mode", "fps_mode", "fps_limit", "metal_hud", "draw_pipe", "readbacks", "preupload",
    "renderer",
];

/// A launcher option as the game reads and writes it: true/false and numbers as JSON, text as it is.
pub fn launcher_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// The game's text for a synced launcher option, typed like its default; None for any other key
/// or a value of the wrong kind.
pub fn launcher_value(key: &str, text: &str, default: &serde_json::Value) -> Option<serde_json::Value> {
    use serde_json::Value;
    if !GAME_SYNCED.contains(&key) {
        return None;
    }
    match default {
        Value::Bool(_) => match text {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ => None,
        },
        Value::Number(_) => text.parse::<u64>().ok().filter(|n| *n <= 1000).map(Value::from),
        Value::String(_) => Some(Value::String(text.to_string())),
        _ => None,
    }
}

/// One "BBSET key=value" line of the game's output (the in-game menu changed a setting).
pub fn parse_game_line(line: &str) -> Option<(String, String)> {
    let (key, value) = line.strip_prefix("BBSET ")?.split_once('=')?;
    let key = key.trim();
    (!key.is_empty() && !key.contains(char::is_whitespace)).then(|| (key.to_string(), value.trim().to_string()))
}

async fn import_once(connection: &mut SqliteConnection, data: &Path) -> db::Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings")
        .fetch_one(&mut *connection)
        .await
        .map_err(db::err)?;
    if count != 0 {
        return Ok(());
    }
    let mut rows: Vec<(&str, String, String)> = Vec::new();
    if let Ok(text) = fs::read_to_string(launcher_json(data)) {
        if let Ok(serde_json::Value::Object(saved)) = serde_json::from_str::<serde_json::Value>(&text) {
            for (key, value) in saved {
                rows.push((LAUNCHER, key, value.to_string()));
            }
        }
    }
    if let Ok(text) = fs::read_to_string(ini_path(data)) {
        for (key, value) in parse_ini(&text) {
            rows.push((GAME, key, value));
        }
    }
    let now = db::now();
    let mut tx = connection.begin_with("BEGIN IMMEDIATE").await.map_err(db::err)?;
    for (scope, key, value) in rows {
        sqlx::query("INSERT OR IGNORE INTO settings (scope,key,value,updated_at) VALUES (?,?,?,?)")
            .bind(scope)
            .bind(key)
            .bind(value)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(db::err)?;
    }
    tx.commit().await.map_err(db::err)
}

/// The stored values of a scope (defaults are the caller's).
pub fn load(data: &Path, scope: &str) -> db::Result<BTreeMap<String, String>> {
    db::block(async {
        let mut connection = db::connect(data).await?;
        import_once(&mut connection, data).await?;
        let rows = sqlx::query("SELECT key, value FROM settings WHERE scope=?")
            .bind(scope)
            .fetch_all(&mut connection)
            .await
            .map_err(db::err)?;
        rows.into_iter()
            .map(|row| Ok((row.try_get("key").map_err(db::err)?, row.try_get("value").map_err(db::err)?)))
            .collect()
    })
}

/// Sets values of a scope in one transaction; None removes the key (its default applies).
pub fn put(data: &Path, scope: &str, values: &[(String, Option<String>)]) -> db::Result<()> {
    db::block(async {
        let mut connection = db::connect(data).await?;
        import_once(&mut connection, data).await?;
        let now = db::now();
        let mut tx = connection.begin_with("BEGIN IMMEDIATE").await.map_err(db::err)?;
        for (key, value) in values {
            match value {
                Some(value) => sqlx::query(
                    "INSERT INTO settings (scope,key,value,updated_at) VALUES (?,?,?,?) \
                     ON CONFLICT(scope,key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
                )
                .bind(scope)
                .bind(key)
                .bind(value)
                .bind(now),
                None => sqlx::query("DELETE FROM settings WHERE scope=? AND key=?").bind(scope).bind(key),
            }
            .execute(&mut *tx)
            .await
            .map_err(db::err)?;
        }
        tx.commit().await.map_err(db::err)
    })
}

/// Writes bbport.ini from the game's stored values (before the game starts): replaced whole, a
/// failed write leaves the old file.
pub fn export_ini(data: &Path, values: &BTreeMap<String, String>) -> Result<(), String> {
    let mut text = String::from("# bbport settings, written by the launcher from its database when the game starts.\n# Change them in the launcher or the in-game menu (F1 / L3+R3).\n");
    for (key, value) in values {
        text.push_str(&format!("{key}={value}\n"));
    }
    let pending = data.join("bbport.ini.pending");
    fs::write(&pending, text).map_err(|e| e.to_string())?;
    fs::rename(pending, ini_path(data)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_lines_and_ini_parse() {
        assert_eq!(parse_game_line("BBSET upscaler=fsr3"), Some(("upscaler".into(), "fsr3".into())));
        assert_eq!(parse_game_line("BBSET sharpness=0.30\r"), Some(("sharpness".into(), "0.30".into())));
        assert_eq!(parse_game_line("BBSET =1"), None);
        assert_eq!(parse_game_line("Runtime: BBSET a=1"), None);
        assert_eq!(parse_game_line("BBSET launcher:fps_limit=60"), Some(("launcher:fps_limit".into(), "60".into())));
        assert_eq!(launcher_value("fps_limit", "60", &serde_json::json!(0)), Some(serde_json::json!(60)));
        assert_eq!(launcher_value("fullscreen", "true", &serde_json::json!(false)), Some(serde_json::json!(true)));
        assert_eq!(launcher_value("fullscreen", "1", &serde_json::json!(false)), None);
        assert_eq!(launcher_value("display", "", &serde_json::json!("")), Some(serde_json::json!("")));
        assert_eq!(launcher_value("extra_env", "A=1", &serde_json::json!("")), None);
        assert_eq!(launcher_text(&serde_json::json!(true)), "true");
        assert_eq!(launcher_text(&serde_json::json!("Fifo")), "Fifo");
        assert_eq!(parse_ini("# c\nupscaler = fsr3\n\nkey.cross=Space\n"),
                   vec![("upscaler".into(), "fsr3".into()), ("key.cross".into(), "Space".into())]);
    }

    #[test]
    fn imports_once_and_round_trips() {
        let root = fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("bb-settings-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(launcher_json(&root), r#"{"fullscreen": true, "fps_limit": 60}"#).unwrap();
        fs::write(ini_path(&root), "# x\nupscaler=fsr3\nshow_fps=1\n").unwrap();
        let game = load(&root, GAME).unwrap();
        assert_eq!(game.get("upscaler").map(String::as_str), Some("fsr3"));
        let launcher = load(&root, LAUNCHER).unwrap();
        assert_eq!(launcher.get("fps_limit").map(String::as_str), Some("60"));
        // The files are imported once: later changes to them are not read again.
        fs::write(ini_path(&root), "upscaler=off\n").unwrap();
        put(&root, GAME, &[("show_fps".into(), Some("0".into())), ("upscaler".into(), None)]).unwrap();
        let game = load(&root, GAME).unwrap();
        assert_eq!(game.get("show_fps").map(String::as_str), Some("0"));
        assert!(!game.contains_key("upscaler"));
        export_ini(&root, &game).unwrap();
        assert!(fs::read_to_string(ini_path(&root)).unwrap().contains("show_fps=0\n"));
        fs::remove_dir_all(&root).unwrap();
    }
}
