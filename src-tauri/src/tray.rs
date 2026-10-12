// SPDX-License-Identifier: GPL-2.0-or-later
// Window/Dock handling adapted from leari-mail/leari 5876482 (MIT, Pablo Souza).
// See docs/licenses/leari-MIT.txt. Game-driven visibility and recovery are project-owned.
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

const TRAY_ID: &str = "game";
struct TrayState {
    running: AtomicBool,
    last_running: AtomicBool,
    open: MenuItem<Wry>,
    stop: MenuItem<Wry>,
}
#[derive(serde::Deserialize)]
pub struct Labels {
    pub show: String,
    pub stop: String,
}

pub fn setup(
    app: &AppHandle,
    on_stop: impl Fn(&AppHandle) + Send + Sync + 'static,
) -> tauri::Result<()> {
    let locale: serde_json::Value =
        serde_json::from_str(include_str!("../../src/i18n/locales/en.json"))?;
    let open = MenuItem::with_id(
        app,
        "show_launcher",
        locale["tray"]["show"].as_str().unwrap_or("Bloodborne"),
        true,
        None::<&str>,
    )?;
    let stop = MenuItem::with_id(
        app,
        "stop_game",
        locale["tray"]["stop"].as_str().unwrap_or("Bloodborne"),
        false,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&open, &stop])?;
    #[cfg(target_os = "macos")]
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;
    #[cfg(not(target_os = "macos"))]
    let icon = app
        .default_window_icon()
        .cloned()
        .unwrap_or(Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?);
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Bloodborne")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show_launcher" => show_launcher(app),
            "stop_game" => on_stop(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                // macOS clicks open the attached menu. On other platforms double-click restores.
                #[cfg(not(target_os = "macos"))]
                show_launcher(tray.app_handle());
                #[cfg(target_os = "macos")]
                let _ = tray;
            }
        })
        .build(app)?;
    tray.set_visible(false)?;
    app.manage(TrayState {
        running: AtomicBool::new(false),
        last_running: AtomicBool::new(false),
        open,
        stop,
    });
    Ok(())
}

pub fn available(app: &AppHandle) -> bool {
    app.try_state::<TrayState>().is_some()
}
pub fn is_running(app: &AppHandle) -> bool {
    app.try_state::<TrayState>()
        .is_some_and(|s| s.running.load(Ordering::SeqCst))
}
pub fn set_labels(app: &AppHandle, labels: Labels) -> tauri::Result<()> {
    if let Some(state) = app.try_state::<TrayState>() {
        state.open.set_text(labels.show)?;
        state.stop.set_text(labels.stop)?;
    }
    Ok(())
}
fn schedule(app: &AppHandle, action: fn(&AppHandle)) {
    let handle = app.clone();
    if let Err(error) = app.run_on_main_thread(move || action(&handle)) {
        eprintln!("Launcher visibility: {error}");
    }
}
// Call while holding the child-state lock: start/exit notifications stay in process order.
// Queued UI work reads the latest state, so a quick exit/relaunch cannot restore a stale window.
pub fn game_changed(app: &AppHandle, running: bool) {
    if let Some(state) = app.try_state::<TrayState>() {
        // A new launch must hide again even if a fast exit/relaunch coalesces in the UI queue.
        state.last_running.store(!running, Ordering::SeqCst);
        state.running.store(running, Ordering::SeqCst);
        schedule(app, sync_current);
    }
}
fn dock_visible(app: &AppHandle, visible: bool) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(if visible {
        tauri::ActivationPolicy::Regular
    } else {
        tauri::ActivationPolicy::Accessory
    })?;
    #[cfg(not(target_os = "macos"))]
    if let Some(window) = app.get_webview_window("main") {
        window.set_skip_taskbar(!visible)?;
    }
    Ok(())
}
fn restore(app: &AppHandle) -> tauri::Result<()> {
    dock_visible(app, true)?;
    if let Some(window) = app.get_webview_window("main") {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}
fn background(app: &AppHandle) -> tauri::Result<()> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or(tauri::Error::Io(std::io::Error::other(
            "Launcher tray icon is unavailable",
        )))?;
    // Never hide the window before a usable tray icon has been made visible.
    tray.set_visible(true)?;
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }
    dock_visible(app, false)
}
fn sync_current(app: &AppHandle) {
    let Some(state) = app.try_state::<TrayState>() else {
        return;
    };
    let running = state.running.load(Ordering::SeqCst);
    if state.last_running.swap(running, Ordering::SeqCst) == running {
        return;
    }
    if let Err(error) = state.stop.set_enabled(running) {
        eprintln!("Tray menu: {error}");
    }
    if running {
        if let Err(error) = background(app) {
            eprintln!("Could not hide launcher: {error}");
            show_now(app);
        }
    } else if let Err(error) = restore(app) {
        // Retain the tray icon if restoration failed, allowing a manual retry.
        eprintln!("Could not restore launcher: {error}");
    } else if let Some(tray) = app.tray_by_id(TRAY_ID) {
        if let Err(error) = tray.set_visible(false) {
            eprintln!("Tray visibility: {error}");
        }
    }
}
fn show_now(app: &AppHandle) {
    if let Some(state) = app.try_state::<TrayState>() {
        state
            .last_running
            .store(state.running.load(Ordering::SeqCst), Ordering::SeqCst);
    }
    if let Err(error) = restore(app) {
        eprintln!("Could not show launcher: {error}");
    }
}
pub fn show_launcher(app: &AppHandle) {
    schedule(app, show_now);
}
fn hide_now(app: &AppHandle) {
    if is_running(app) {
        if let Err(error) = background(app) {
            eprintln!("Could not hide launcher: {error}");
            show_now(app);
        }
    }
}
pub fn hide_while_running(app: &AppHandle) {
    schedule(app, hide_now);
}
