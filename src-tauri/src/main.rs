// SPDX-License-Identifier: GPL-2.0-or-later
//! bloodborne_mac launcher: the Tauri backend behind the React interface (launcher/app/src).
//!
//! It keeps two files in the data directory (~/Library/Application Support/bloodborne_mac):
//! launcher.json (start-up options passed to run.sh as environment variables) and bbport.ini
//! (the port's settings, shared with the in-game menu; lines it does not change are kept).
//! Mods, third-party patches and the game check come from scripts/launcher_api.py (the same code
//! run.sh uses), controllers from bb-gpu-capabilities (--gamepads, --read-input). The game runs
//! as `bash run.sh` with the app's own bash and Python; its output is sent to the window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod saves;
mod profiles;
mod database;

use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

/// Where the launcher finds the game's programs and scripts.
#[derive(Clone, Serialize)]
struct Paths {
    /// run.sh, scripts/, patches/ (Contents/Resources/game, or a source checkout).
    game_root: PathBuf,
    bash: PathBuf,
    python: PathBuf,
    probe: PathBuf,
    capabilities: PathBuf,
    data: PathBuf,
    /// Packaged in an app bundle (else a development run from a checkout).
    bundled: bool,
}

impl Paths {
    fn find() -> Paths {
        let exe = std::env::current_exe().unwrap_or_default();
        let macos = exe.parent().map(Path::to_path_buf).unwrap_or_default();
        let data = dirs_data().join("bloodborne_mac");
        let resources = macos.parent().map(|c| c.join("Resources")).unwrap_or_default();
        if resources.join("game/run.sh").is_file() {
            return Paths {
                game_root: resources.join("game"),
                bash: macos.join("bash"),
                python: resources.join("python/bin/python3"),
                probe: macos.join("bb-probe"),
                capabilities: macos.join("bb-gpu-capabilities"),
                data,
                bundled: true,
            };
        }
        // Development (npm run tauri dev): the checkout's build (the runtime/ submodule,
        // bloodborne-runtime, or BB_REPO), Homebrew's bash, any Python 3.
        let root = std::env::var_os("BB_REPO").map(PathBuf::from).unwrap_or_else(|| {
            let mut dir = macos.clone();
            loop {
                if dir.join("runtime/run.sh").is_file() {
                    return dir.join("runtime");
                }
                if dir.join("run.sh").is_file() || !dir.pop() {
                    return dir;
                }
            }
        });
        let bash = ["/opt/homebrew/bin/bash", "/usr/local/bin/bash"]
            .iter()
            .map(PathBuf::from)
            .find(|p| p.is_file())
            .unwrap_or_else(|| PathBuf::from("/bin/bash"));
        Paths {
            probe: root.join("out/bb-probe"),
            capabilities: root.join("out/bb-gpu-capabilities"),
            game_root: root,
            bash,
            python: PathBuf::from("python3"),
            data,
            bundled: false,
        }
    }
}

fn dirs_data() -> PathBuf {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Library/Application Support"))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Start-up options (launcher.json) and their defaults; unknown keys are kept.
fn default_settings() -> Map<String, Value> {
    let v = json!({
        "game_dir": "",
        "user_dir": "",
        "save_profile": "original",
        "mods_dir": "",
        "mods_enabled": true,
        "patches_dir": "",
        "language": "1",
        "fullscreen": false,
        "display": "",
        "hdr": false,
        "present_mode": "Fifo",
        "gamepad": "",
        "gamepad_name": "",
        "fps_mode": "uncap",
        "fps_limit": 0,
        "draw_pipe": "",
        "readbacks": "",
        "preupload": "",
        "frame_stats": true,
        "save_log": false,
        "crash_diag": false,
        "gpu_profile": false,
        "extra_env": "",
        "ui_language": ""
    });
    v.as_object().cloned().unwrap_or_default()
}

/// bbport.ini values the launcher shows before the file has them (macOS defaults).
fn default_ini() -> BTreeMap<String, String> {
    [
        ("upscaler", "fsr3"),
        ("preset", "2"),
        ("sharpen", "1"),
        ("sharpness", "0.50"),
        ("object_motion", "0"),
        ("show_fps", "1"),
        ("output_res", "1920x1080"),
        ("model_lod", "0"),
        ("live_resolution", "0"),
        ("effect_chromatic_aberration", "1"),
        ("effect_dof", "1"),
        ("effect_motion_blur", "1"),
        ("effect_ssao", "1"),
        ("effect_game_aa", "1"),
        ("effect_dynamic_shadows", "1"),
        ("effect_ssr", "0"),
        ("skip_intro", "0"),
        ("debug_camera", "0"),
        ("debug_menu", "0"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

struct Game {
    child: Option<Child>,
}

#[derive(Clone)]
struct AppState {
    paths: Paths,
    game: Arc<Mutex<Game>>,
}

fn settings_file(p: &Paths) -> PathBuf {
    p.data.join("launcher.json")
}
fn ini_file(p: &Paths) -> PathBuf {
    p.data.join("bbport.ini")
}

fn load_settings(p: &Paths) -> Map<String, Value> {
    let mut settings = default_settings();
    if let Ok(text) = fs::read_to_string(settings_file(p)) {
        if let Ok(Value::Object(saved)) = serde_json::from_str::<Value>(&text) {
            settings.extend(saved);
        }
    }
    settings
}

fn read_ini_lines(p: &Paths) -> Vec<String> {
    fs::read_to_string(ini_file(p))
        .map(|t| t.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

fn ini_key(line: &str) -> Option<&str> {
    if line.trim_start().starts_with('#') {
        return None;
    }
    line.split_once('=').map(|(k, _)| k.trim())
}

fn load_ini(p: &Paths) -> BTreeMap<String, String> {
    let mut values = default_ini();
    for line in read_ini_lines(p) {
        if let Some((k, v)) = line.split_once('=') {
            if !k.trim_start().starts_with('#') {
                values.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    values
}

fn str_setting(s: &Map<String, Value>, key: &str) -> String {
    match s.get(key) {
        Some(Value::String(v)) => v.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => (if *b { "1" } else { "0" }).to_string(),
        _ => String::new(),
    }
}

fn bool_setting(s: &Map<String, Value>, key: &str) -> bool {
    matches!(s.get(key), Some(Value::Bool(true)))
}

fn expand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn mods_dir(p: &Paths, s: &Map<String, Value>) -> PathBuf {
    let dir = str_setting(s, "mods_dir");
    if dir.is_empty() { p.data.join("mods") } else { expand(&dir) }
}

fn patches_dir(p: &Paths, s: &Map<String, Value>) -> PathBuf {
    let dir = str_setting(s, "patches_dir");
    if dir.is_empty() { p.data.join("patches") } else { expand(&dir) }
}

/// scripts/launcher_api.py with the app's Python.
fn api(p: &Paths, args: &[&str]) -> Result<Value, String> {
    let out = Command::new(&p.python)
        .arg(p.game_root.join("scripts/launcher_api.py"))
        .args(args)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .output()
        .map_err(|e| format!("Python: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
}

async fn blocking<T: Send + 'static>(operation: impl FnOnce() -> Result<T,String> + Send + 'static) -> Result<T,String> {
    tauri::async_runtime::spawn_blocking(operation).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn database_path(state: State<'_, AppState>) -> Result<String,String> {
    let data=state.paths.data.clone();
    blocking(move || { database::block(async { database::connect(&data).await?; Ok(()) })?; Ok(database::path(&data).to_string_lossy().into()) }).await
}

// --- commands --------------------------------------------------------------------------------

#[tauri::command]
async fn load_config(state: State<'_ , AppState>) -> Result<Value, String> {
    let state=state.inner().clone();
    blocking(move || {
    let p = &state.paths;
    let s = load_settings(p);
    Ok(json!({
        "settings": s,
        "ini": load_ini(p),
        "paths": {
            "data": p.data,
            "mods": mods_dir(p, &s),
            "patches": patches_dir(p, &s),
            "logs": p.data.join("logs"),
            "saves": selected_user_dir(p).unwrap_or_default(),
        },
        "bundled": p.bundled,
        "version": env!("CARGO_PKG_VERSION"),
        "running": state.game.lock().map(|g| g.child.is_some()).unwrap_or(false),
    }))
    }).await
}

#[tauri::command]
fn save_settings(state: State<AppState>, settings: Map<String, Value>) -> Result<(), String> {
    let game = state.game.lock().map_err(|e| e.to_string())?;
    if settings.contains_key("save_profile") || settings.contains_key("user_dir") { return Err("Manage profile ownership in the Profiles tab.".into()); }
    let _ = &game; // Serialize settings writes with profile changes and launch.
    let mut merged = load_settings(&state.paths);
    merged.extend(settings);
    write_settings(&state.paths, merged)
}

fn write_settings(p: &Paths, settings: Map<String, Value>) -> Result<(), String> {
    fs::create_dir_all(&p.data).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(&Value::Object(settings)).map_err(|e| e.to_string())?;
    let pending = p.data.join("launcher.json.pending");
    fs::write(&pending, text + "\n").map_err(|e| e.to_string())?;
    fs::rename(pending, settings_file(p)).map_err(|e| e.to_string())
}
fn original_user_dir(p: &Paths, s: &Map<String, Value>) -> PathBuf {
    let custom = str_setting(s, "user_dir");
    if custom.is_empty() { p.data.join("user") } else { expand(&custom) }
}
fn selected_user_dir(p: &Paths) -> Result<PathBuf, String> {
    Ok(profiles::selected(&p.data)?.path)
}
fn saves_idle(game: &Game) -> Result<(), String> {
    if game.child.is_some() { return Err("Stop the game before changing or copying saves.".into()); }
    // Also exclude games started outside this launcher, including isolated benchmark runs.
    let status = Command::new("pgrep").args(["-x", "bb-probe"]).stdout(Stdio::null()).status().map_err(|e| e.to_string())?;
    if status.success() { return Err("Stop the running game before changing or copying saves.".into()); }
    if status.code() != Some(1) { return Err("Could not check whether the game is running.".into()); }
    Ok(())
}
#[tauri::command]
async fn list_profiles(state: State<'_, AppState>) -> Result<profiles::ProfileList, String> {
    let state=state.inner().clone();
    blocking(move || {
    let _game = state.game.lock().map_err(|e| e.to_string())?;
    let s = load_settings(&state.paths);
    profiles::list(&state.paths.data, &original_user_dir(&state.paths, &s))
    }).await
}
#[tauri::command]
async fn change_profile(state: State<'_, AppState>, action: String, id: String, name: Option<String>, source: Option<String>) -> Result<(), String> {
    let state=state.inner().clone();
    blocking(move || {
    let game = state.game.lock().map_err(|e| e.to_string())?;
    saves_idle(&game)?;
    let p = &state.paths;
    if action == "initialize" {
        let s = load_settings(p);
        profiles::initialize(&p.data, &original_user_dir(p, &s), &str_setting(&s, "save_profile"), name.as_deref().unwrap_or_default())
    } else { profiles::change(&p.data, &action, &id, name.as_deref(), source.as_deref()) }
    }).await
}
#[tauri::command]
async fn list_saves(state: State<'_, AppState>) -> Result<Value, String> {
    let state=state.inner().clone();
    blocking(move || {
    let _game = state.game.lock().map_err(|e| e.to_string())?;
    let selected = profiles::selected(&state.paths.data)?;
    Ok(json!({"active": selected.id, "profiles": [selected]}))
    }).await
}
#[tauri::command]
async fn change_save(state: State<'_, AppState>, action: String, id: String, backup: Option<String>) -> Result<(), String> {
    let state=state.inner().clone();
    blocking(move || {
    let game = state.game.lock().map_err(|e| e.to_string())?;
    saves_idle(&game)?;
    let p = &state.paths;
    save_operation(p, &id, &action, backup.as_deref())
    }).await
}

fn save_operation(p: &Paths, id: &str, action: &str, backup: Option<&str>) -> Result<(), String> {
    if action != "backup" && action != "restore" { return Err("Manage identities in the Profiles tab.".into()); }
    let registry=profiles::require(&p.data)?;
    if profiles::selected(&p.data)?.id != id { return Err("Choose the owning profile before managing its saves.".into()); }
    let operation=profiles::operation(&p.data, Some(id), action)?;
    let result=(|| {
        if action == "backup" { saves::backup(&p.data, &registry.original_user_dir, id)?; }
        else { saves::restore(&p.data, &registry.original_user_dir, id, backup.unwrap_or_default())?; }
        profiles::list(&p.data, &registry.original_user_dir)?;
        Ok(())
    })();
    profiles::finish(&p.data, &operation, &result)?;
    result
}

/// Sets bbport.ini keys (null removes the line: the default applies), keeping the other lines.
#[tauri::command]
fn save_ini(state: State<AppState>, values: BTreeMap<String, Option<String>>) -> Result<(), String> {
    let p = &state.paths;
    fs::create_dir_all(&p.data).map_err(|e| e.to_string())?;
    let mut lines = read_ini_lines(p);
    for (key, value) in &values {
        let pos = lines.iter().position(|l| ini_key(l) == Some(key.as_str()));
        match (pos, value) {
            (Some(i), Some(v)) => lines[i] = format!("{key}={v}"),
            (Some(i), None) => {
                lines.remove(i);
            }
            (None, Some(v)) => lines.push(format!("{key}={v}")),
            (None, None) => {}
        }
    }
    fs::write(ini_file(p), lines.join("\n") + "\n").map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_game(state: State<'_, AppState>, dir: String) -> Result<Value, String> {
    if dir.is_empty() {
        return Ok(json!({"problem": "Choose the game folder.", "game": null}));
    }
    api(&state.paths, &["check", &expand(&dir).to_string_lossy()])
}

#[tauri::command]
async fn list_mods(state: State<'_, AppState>) -> Result<Value, String> {
    let p = &state.paths;
    let s = load_settings(p);
    let dir = mods_dir(p, &s);
    let _ = fs::create_dir_all(&dir);
    let mut v = api(p, &["mods", &dir.to_string_lossy(), &p.data.join("mods.json").to_string_lossy()])?;
    v["dir"] = json!(dir);
    Ok(v)
}

#[tauri::command]
fn save_mods(state: State<AppState>, order: Vec<String>, disabled: Vec<String>) -> Result<(), String> {
    let p = &state.paths;
    fs::create_dir_all(&p.data).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(&json!({"order": order, "disabled": disabled}))
        .map_err(|e| e.to_string())?;
    fs::write(p.data.join("mods.json"), text + "\n").map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_patches(state: State<'_, AppState>) -> Result<Value, String> {
    let p = &state.paths;
    let s = load_settings(p);
    let dir = patches_dir(p, &s);
    let _ = fs::create_dir_all(&dir);
    let mut v = api(p, &["patches", &dir.to_string_lossy(), &p.data.join("patches.json").to_string_lossy()])?;
    v["dir"] = json!(dir);
    Ok(v)
}

/// patches.json: the switches that differ from each file's default; choices for patches not
/// listed now (another folder) are kept.
#[tauri::command]
fn save_patches(state: State<AppState>, shown: Vec<String>, enabled: Vec<String>, disabled: Vec<String>) -> Result<(), String> {
    let p = &state.paths;
    let path = p.data.join("patches.json");
    let old: Value = fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({}));
    let keep = |key: &str| -> Vec<String> {
        old.get(key)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).filter(|k| !shown.iter().any(|s| s == k)).map(str::to_string).collect())
            .unwrap_or_default()
    };
    let mut en = keep("enabled");
    en.extend(enabled);
    en.sort();
    en.dedup();
    let mut dis = keep("disabled");
    dis.extend(disabled);
    dis.sort();
    dis.dedup();
    fs::create_dir_all(&p.data).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(&json!({"enabled": en, "disabled": dis})).map_err(|e| e.to_string())?;
    fs::write(path, text + "\n").map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct Gamepad {
    guid: String,
    name: String,
}

#[tauri::command]
async fn list_gamepads(state: State<'_, AppState>) -> Result<Vec<Gamepad>, String> {
    let out = Command::new(&state.paths.capabilities)
        .arg("--gamepads")
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(g, n)| Gamepad { guid: g.to_string(), name: n.to_string() })
        .collect())
}

#[derive(Serialize)]
struct Display {
    name: String,
    size: String,
    primary: bool,
}

/// The monitors in SDL's order, as the game sees them: "name<TAB>WxH<TAB>primary" lines.
#[tauri::command]
async fn list_displays(state: State<'_, AppState>) -> Result<Vec<Display>, String> {
    let out = Command::new(&state.paths.capabilities)
        .arg("--displays")
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let mut f = l.split('\t');
            Some(Display { name: f.next()?.to_string(), size: f.next()?.to_string(), primary: f.next()? == "1" })
        })
        .collect())
}

/// One key or button for a control: bb-gpu-capabilities opens a small window and prints
/// "key <name>" or "pad <name>"; None when cancelled.
#[tauri::command]
async fn read_input(state: State<'_, AppState>, kind: String) -> Result<Option<String>, String> {
    let caps = state.paths.capabilities.clone();
    let out = tauri::async_runtime::spawn_blocking(move || Command::new(caps).args(["--read-input", &kind]).output())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(text.split_once(' ').map(|(_, name)| name.to_string()))
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    let path = expand(&path);
    let _ = fs::create_dir_all(&path);
    Command::new("open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
}

// GUI names may be long; keep the guest value within a 16-byte caller buffer including NUL.
fn guest_profile_name(name: &str) -> String {
    let end = name.char_indices().map(|(start,c)|start+c.len_utf8()).take_while(|end|*end<=15).last().unwrap_or(0);
    name[..end].to_string()
}

fn game_environment(p: &Paths, s: &Map<String, Value>) -> Result<Vec<(String, String)>, String> {
    let mut env: Vec<(String, String)> = Vec::new();
    let mut set = |k: &str, v: String| env.push((k.to_string(), v));
    let path_dirs = [p.bash.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default(),
                     "/usr/bin:/bin:/usr/sbin:/sbin".to_string()];
    set("PATH", path_dirs.join(":"));
    set("BB_PREBUILT", "1".into());
    set("BB_PROBE", p.probe.to_string_lossy().into());
    set("BB_DATA_DIR", p.data.to_string_lossy().into());
    set("PYTHON", p.python.to_string_lossy().into());
    set("PYTHONDONTWRITEBYTECODE", "1".into());
    set("BB_GAME_DIR", expand(&str_setting(s, "game_dir")).to_string_lossy().into());
    set("BB_MODS_DIR", mods_dir(p, s).to_string_lossy().into());
    set("BB_MODS_CONFIG", p.data.join("mods.json").to_string_lossy().into());
    set("BB_MODS_ENABLED", (if bool_setting(s, "mods_enabled") { "1" } else { "0" }).into());
    set("BB_PATCHES_DIR", patches_dir(p, s).to_string_lossy().into());
    set("BB_PATCHES_CONFIG", p.data.join("patches.json").to_string_lossy().into());
    set("BB_LANGUAGE", str_setting(s, "language"));
    // The in-game menu follows the launcher's language; the game matches regional codes the
    // way the launcher does (gpu/shim/bbport_text.cpp).
    let ui = str_setting(s, "ui_language");
    set("BB_UI_LANGUAGE", if ui.is_empty() { system_language() } else { ui });
    set("BB_FULLSCREEN", (if bool_setting(s, "fullscreen") { "1" } else { "0" }).into());
    set("BB_PRESENT_MODE", str_setting(s, "present_mode"));
    // display: the monitor's name (BB_DISPLAY matches part of it), so it survives a new order.
    for (key, var) in [("gamepad", "BB_GAMEPAD"), ("display", "BB_DISPLAY"), ("draw_pipe", "BB_DRAW_PIPE"),
                       ("readbacks", "BB_READBACKS"), ("preupload", "BB_PREUPLOAD")] {
        let v = str_setting(s, key);
        if !v.is_empty() {
            set(var, v);
        }
    }
    if bool_setting(s, "hdr") {
        set("BB_HDR", "1".into());
    }
    set("BB_FPS", str_setting(s, "fps_mode"));
    let limit = s.get("fps_limit").and_then(Value::as_u64).unwrap_or(0);
    if limit > 0 {
        set("BB_FPS_LIMIT", limit.to_string());
    }
    if bool_setting(s, "frame_stats") {
        set("BB_FRAME_STATS", "1".into());
    }
    if bool_setting(s, "save_log") {
        set("BB_SAVE_LOG", "1".into());
    }
    if bool_setting(s, "crash_diag") {
        set("BB_FREE_CHECK", "1".into());
        set("BB_WRITE_LOG", "1".into());
    }
    if bool_setting(s, "gpu_profile") {
        set("BB_GPU_PROFILE", "1".into());
    }
    for item in str_setting(s, "extra_env").split_whitespace() {
        if let Some((k, v)) = item.split_once('=') {
            set(k, v.to_string());
        }
    }
    // The selected profile wins over inherited and extra environment save paths. GPU caches
    // stay in the original user directory, shared by all profiles.
    set("BB_USER_DIR", selected_user_dir(p)?.to_string_lossy().into());
    set("BB_GPU_USER_DIR", profiles::require(&p.data)?.original_user_dir.to_string_lossy().into());
    set("BB_USER_NAME", guest_profile_name(&profiles::selected(&p.data)?.name));
    Ok(env)
}

/// The system's preferred language ("pt-BR"): an app started from Finder has no LANG.
fn system_language() -> String {
    Command::new("defaults")
        .args(["read", "-g", "AppleLanguages"])
        .output()
        .ok()
        .and_then(|o| {
            // "(\n    \"pt-BR\",\n    en\n)": the first item, quoted or not
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().trim_end_matches(',').trim_matches('"'))
                .find(|l| !l.is_empty() && *l != "(" && *l != ")")
                .map(str::to_string)
        })
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default()
}

#[tauri::command]
async fn launch(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let state=state.inner().clone();
    blocking(move || start_game(&app, &state)).await
}

fn start_game(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let app = app.clone();
    let p = state.paths.clone();
    let mut game = state.game.lock().map_err(|e| e.to_string())?;
    if game.child.is_some() {
        return Err("The game is already running.".into());
    }
    saves_idle(&game)?;
    let s = load_settings(&p);
    selected_user_dir(&p)?;
    let registry = profiles::require(&p.data)?;
    fs::create_dir_all(&registry.original_user_dir).map_err(|e| e.to_string())?;
    // Back up the complete set before launch. Failure prevents starting without a recovery copy.
    save_operation(&p, &registry.active, "backup", None)?;
    fs::create_dir_all(p.data.join("logs")).map_err(|e| e.to_string())?;
    let log_path = p.data.join("logs/last.log");
    let mut cmd = Command::new(&p.bash);
    cmd.arg("run.sh")
        .current_dir(&p.game_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in game_environment(&p, &s)? {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().map_err(|e| format!("Could not start the game: {e}"))?;
    let log = Arc::new(Mutex::new(fs::File::create(&log_path).ok()));
    for stream in [child.stdout.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
                   child.stderr.take().map(|s| Box::new(s) as Box<dyn std::io::Read + Send>)]
        .into_iter()
        .flatten()
    {
        let app = app.clone();
        let log = log.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                if let Ok(mut f) = log.lock() {
                    if let Some(f) = f.as_mut() {
                        use std::io::Write;
                        let _ = writeln!(f, "{line}");
                    }
                }
                let _ = app.emit("game-output", line);
            }
        });
    }
    game.child = Some(child);
    let _ = app.emit("game-state", json!({"running": true}));
    // Waits for the game to end without holding the lock.
    let shared = state.game.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let mut g = match shared.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let status = match g.child.as_mut().map(Child::try_wait) {
            Some(Ok(Some(status))) => status,
            Some(Ok(None)) => continue,
            _ => {
                g.child = None;
                return;
            }
        };
        g.child = None;
        let _ = app.emit("game-state", json!({"running": false, "code": status.code()}));
        return;
    });
    Ok(())
}

#[tauri::command]
fn stop(state: State<AppState>) -> Result<(), String> {
    let game = state.game.lock().map_err(|e| e.to_string())?;
    if let Some(child) = game.child.as_ref() {
        // SIGTERM: run.sh execs the game, which quits cleanly on it.
        let _ = Command::new("kill").arg(child.id().to_string()).status();
    }
    Ok(())
}

fn main() {
    let paths = Paths::find();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _arguments, _cwd| {
            if let Some(window)=app.get_webview_window("main") { let _=window.show(); let _=window.set_focus(); }
        }))
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { paths, game: Arc::new(Mutex::new(Game { child: None })) })
        // `open Bloodborne.app --args --play`: start the game at once (shortcuts, scripted tests).
        .setup(|app| {
            if let Err(error) = profiles::recover_operations(&app.state::<AppState>().paths.data) { eprintln!("Profile recovery: {error}"); }
            if std::env::args().any(|a| a == "--play") {
                let handle = app.handle().clone();
                let state = app.state::<AppState>();
                if let Err(e) = start_game(&handle, &state) {
                    eprintln!("--play: {e}");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_config, save_settings, save_ini, check_game, list_mods, save_mods, list_patches,
            save_patches, list_gamepads, list_displays, read_input, open_path, launch, stop, list_saves, change_save, list_profiles, change_profile, database_path
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if let Some(state) = window.app_handle().try_state::<AppState>() {
                    if let Ok(game) = state.game.lock() {
                        if let Some(child) = game.child.as_ref() {
                            let _ = Command::new("kill").arg(child.id().to_string()).status();
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running the launcher");
}

#[cfg(test)]
mod save_tests {
    use super::*;

    #[test]
    fn guest_profile_name_fits_small_buffers_without_splitting_utf8() {
        assert_eq!(guest_profile_name("Pablo"), "Pablo");
        let long=guest_profile_name("日本語の長いプロフィール名");
        assert!(long.len() < 16);
        assert!("日本語の長いプロフィール名".starts_with(&long));
    }
    #[test]
    fn save_changes_are_rejected_while_launcher_child_runs() {
        let child = Command::new("sleep").arg("30").spawn().unwrap();
        let mut game = Game { child: Some(child) };
        assert!(saves_idle(&game).unwrap_err().contains("Stop the game"));
        game.child.as_mut().unwrap().kill().unwrap();
        game.child.as_mut().unwrap().wait().unwrap();
    }

    #[test]
    fn selected_profile_controls_launch_and_shares_only_gpu_cache_path() {
        let root = fs::canonicalize(std::env::temp_dir()).unwrap().join(format!(
            "bb-save-launch-test-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();
        let p = Paths { game_root: root.clone(), bash: PathBuf::from("bash"), python: PathBuf::from("python3"),
            probe: root.join("bb-probe"), capabilities: root.join("capabilities"), data: root.clone(), bundled: false };
        let mut settings = default_settings();
        assert!(selected_user_dir(&p).is_err());
        profiles::initialize(&root, &root.join("user"), "original", "First").unwrap();
        assert_eq!(selected_user_dir(&p).unwrap(), root.join("user"));
        let id = saves::create(&root, &root.join("user"), "Synthetic", None).unwrap();
        assert!(save_operation(&p, &id, "backup", None).is_err());
        profiles::change(&root, "select", &id, None, None).unwrap();
        save_operation(&p, &id, "backup", None).unwrap();
        assert_eq!(profiles::selected(&root).unwrap().backups.len(), 1);
        settings.insert("extra_env".into(), json!("BB_USER_DIR=/wrong BB_GPU_USER_DIR=/wrong BB_USER_NAME=wrong"));
        let env: BTreeMap<_,_> = game_environment(&p, &settings).unwrap().into_iter().collect();
        assert_eq!(env["BB_USER_DIR"], root.join("save-profiles").join(&id).join("user").to_string_lossy());
        assert_eq!(env["BB_GPU_USER_DIR"], root.join("user").to_string_lossy());
        assert_eq!(env["BB_USER_NAME"], "Synthetic");
        // Configuration and original save files are never moved by profile creation/selection.
        assert!(!root.join("user").exists());
        fs::remove_dir_all(root.join("save-profiles").join(&id)).unwrap();
        assert!(game_environment(&p, &settings).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
