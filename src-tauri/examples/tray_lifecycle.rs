// SPDX-License-Identifier: GPL-2.0-or-later
//! Native GUI smoke test. Uses only short-lived synthetic processes and about:blank.
//! No game, profile database, saves, configuration or screenshots are involved.
#[path = "../src/tray.rs"]
mod tray;
use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn wait_for(app: &tauri::AppHandle, visible: bool) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let window = app
            .get_webview_window("main")
            .ok_or("Test window missing")?;
        if window.is_visible().map_err(|e| e.to_string())? == visible {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err(format!("Launcher did not become visible={visible}"))
}
fn exercise(app: &tauri::AppHandle) -> Result<(), String> {
    wait_for(app, true)?;
    tray::set_labels(
        app,
        tray::Labels {
            show: "Show test launcher".into(),
            stop: "Stop test process".into(),
        },
    )
    .map_err(|e| e.to_string())?;
    for code in [0, 7] {
        let mut child = Command::new("sh")
            .args(["-c", &format!("sleep 1; exit {code}")])
            .spawn()
            .map_err(|e| e.to_string())?;
        tray::game_changed(app, true);
        wait_for(app, false)?;
        if code == 0 {
            tray::show_launcher(app);
            wait_for(app, true)?;
            app.get_webview_window("main")
                .ok_or("Test window missing")?
                .close()
                .map_err(|e| e.to_string())?;
            wait_for(app, false)?;
        }
        let result = child.wait().map_err(|e| e.to_string())?;
        if result.code() != Some(code) {
            return Err("Unexpected synthetic exit code".into());
        }
        tray::game_changed(app, false);
        wait_for(app, true)?;
        println!("PASS: process exit {code} restores the launcher");
    }
    // A quick restart after manually reopening must hide again, even when queued
    // finish/start notifications collapse to the same running value.
    tray::game_changed(app, true);
    wait_for(app, false)?;
    tray::show_launcher(app);
    wait_for(app, true)?;
    tray::game_changed(app, false);
    tray::game_changed(app, true);
    wait_for(app, false)?;
    tray::game_changed(app, false);
    wait_for(app, true)?;
    println!("PASS: rapid restart hides a manually reopened launcher again");
    let mut child = Command::new("sleep")
        .arg("30")
        .spawn()
        .map_err(|e| e.to_string())?;
    tray::game_changed(app, true);
    wait_for(app, false)?;
    child.kill().map_err(|e| e.to_string())?;
    child.wait().map_err(|e| e.to_string())?;
    tray::game_changed(app, false);
    wait_for(app, true)?;
    println!("PASS: killed process restores launcher; manual show/close, menu labels and idle state work");
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = tauri::generate_context!();
    context.config_mut().identifier = "io.bloodborne-native.tray-lifecycle-test".into();
    context.config_mut().product_name = Some("Bloodborne tray test".into());
    context.config_mut().app.windows.clear();
    tauri::Builder::default()
        .setup(|app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External("about:blank".parse()?))
                .title("Launcher tray test — no game or saves")
                .inner_size(460.0, 180.0)
                .build()?;
            tray::setup(app.handle(), |_| {})?;
            if !tray::available(app.handle()) {
                return Err("Test tray unavailable".into());
            }
            let handle = app.handle().clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(300));
                match exercise(&handle) {
                    Ok(()) => handle.exit(0),
                    Err(error) => {
                        eprintln!("FAIL: {error}");
                        handle.exit(1);
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if tray::is_running(window.app_handle()) {
                    api.prevent_close();
                    tray::hide_while_running(window.app_handle());
                }
            }
        })
        .run(context)?;
    Ok(())
}
