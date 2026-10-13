// SPDX-License-Identifier: GPL-2.0-or-later
//! Opt-in, synthetic notification helper. No profile database, game files or launcher startup.
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, Read},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{Emitter, Manager};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Notice {
    v: u8,
    seq: u64,
    id: u8,
    x: i32,
    y: i32,
    duration_ms: u32,
    #[serde(default = "default_visible")]
    visible: bool,
}
fn default_visible() -> bool {
    true
}
fn parse(line: &[u8]) -> Result<Notice, String> {
    if line.len() > 4096 {
        return Err("message exceeds 4096 bytes".into());
    }
    let n: Notice = serde_json::from_slice(line).map_err(|e| e.to_string())?;
    if n.v != 1
        || n.seq == 0
        || n.id >= 128
        || n.x.abs_diff(0) > 100_000
        || n.y.abs_diff(0) > 100_000
        || !(1000..=10_000).contains(&n.duration_ms)
    {
        return Err("invalid notification fields".into());
    }
    Ok(n)
}
#[tauri::command]
fn overlay_ready(window: tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "overlay" {
        return Err("wrong window".into());
    }
    println!("{{\"v\":1,\"type\":\"ready\"}}");
    Ok(())
}
#[cfg(target_os = "macos")]
fn configure_native(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    let raw = window.ns_window()?;
    // Tauri owns this NSWindow; called on its main event-loop thread while the window is alive.
    let native = unsafe { &*raw.cast::<NSWindow>() };
    native.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary,
    );
    native.setHidesOnDeactivate(false);
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn configure_native(_window: &tauri::WebviewWindow) -> tauri::Result<()> {
    Ok(())
}
#[cfg(target_os = "macos")]
fn show_without_focus(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    use objc2_app_kit::NSWindow;
    // Tauri/Tao show() makes the window key and activates the app on macOS.
    // Ordering this owned window directly preserves the current app and game input.
    let native = unsafe { &*window.ns_window()?.cast::<NSWindow>() };
    native.orderFrontRegardless();
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn show_without_focus(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    window.show()
}

fn main() {
    if !std::env::args().any(|a| a == "--prototype") {
        eprintln!("This synthetic helper requires --prototype and JSON lines on stdin.");
        std::process::exit(2);
    }
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    context.config_mut().identifier = "io.github.bloodborne-native.overlay-prototype".into();
    let generation = Arc::new(AtomicU64::new(0));
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![overlay_ready])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("overlay.html".into()),
            )
            .title("Bloodborne notification prototype")
            .inner_size(460.0, 140.0)
            .incognito(true)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .resizable(false)
            .focused(false)
            .focusable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()?;
            window.set_ignore_cursor_events(true)?;
            configure_native(&window)?;
            let handle = app.handle().clone();
            let generation = generation.clone();
            let (sender, receiver) = std::sync::mpsc::sync_channel::<Notice>(8);
            // A bounded input queue and one expiry worker; no thread is created per notice.
            std::thread::spawn(move || {
                let mut input = std::io::stdin().lock();
                let mut last = 0;
                loop {
                    let mut line = Vec::new();
                    match input.by_ref().take(4097).read_until(b'\n', &mut line) {
                        Ok(0) => break,
                        Ok(_) if line.len() > 4096 => {
                            eprintln!("Overlay: oversized input; closing");
                            break;
                        }
                        Err(error) => {
                            eprintln!("Overlay input: {error}");
                            break;
                        }
                        _ => {}
                    }
                    let notice = match parse(&line) {
                        Ok(n) if n.seq > last => n,
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Overlay input: {e}");
                            continue;
                        }
                    };
                    last = notice.seq;
                    if let Err(error) = sender.try_send(notice) {
                        match error {
                            std::sync::mpsc::TrySendError::Full(_) => {
                                eprintln!("Overlay: input queue full; notification dropped")
                            }
                            std::sync::mpsc::TrySendError::Disconnected(_) => break,
                        }
                    }
                }
                // Dropping sender closes the owned session. No unrelated process is terminated.
            });
            std::thread::spawn(move || {
                let mut deadline: Option<std::time::Instant> = None;
                loop {
                    let received = match deadline {
                        Some(at) => receiver
                            .recv_timeout(at.saturating_duration_since(std::time::Instant::now())),
                        None => receiver
                            .recv()
                            .map_err(|_| std::sync::mpsc::RecvTimeoutError::Disconnected),
                    };
                    match received {
                        Ok(notice) => {
                            let seq = notice.seq;
                            generation.store(seq, Ordering::SeqCst);
                            deadline = Some(
                                std::time::Instant::now()
                                    + Duration::from_millis(notice.duration_ms as u64),
                            );
                            if !notice.visible {
                                deadline = None;
                                let app = handle.clone();
                                let _ = handle.run_on_main_thread(move || {
                                    if let Some(window) = app.get_webview_window("overlay") {
                                        let _ = window.hide();
                                    }
                                });
                                continue;
                            }
                            let app = handle.clone();
                            if handle
                                .run_on_main_thread(move || {
                                    if let Some(window) = app.get_webview_window("overlay") {
                                        let result = window
                                            .set_position(tauri::LogicalPosition::new(
                                                notice.x as f64,
                                                notice.y as f64,
                                            ))
                                            .and_then(|()| {
                                                window.emit("prototype-notification", &notice)
                                            })
                                            .and_then(|()| show_without_focus(&window));
                                        match result {
                                            Ok(()) => println!(
                                                "{{\"v\":1,\"type\":\"shown\",\"seq\":{seq}}}"
                                            ),
                                            Err(error) => eprintln!("Overlay window: {error}"),
                                        }
                                    }
                                })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            deadline = None;
                            let seq = generation.load(Ordering::SeqCst);
                            let current = generation.clone();
                            let app = handle.clone();
                            let _ = handle.run_on_main_thread(move || {
                                if current.load(Ordering::SeqCst) == seq {
                                    if let Some(window) = app.get_webview_window("overlay") {
                                        let _ = window.hide();
                                    }
                                }
                            });
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                handle.exit(0);
            });
            Ok(())
        })
        .run(context)
        .expect("overlay prototype failed");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_protocol() {
        assert!(parse(br#"{"v":1,"seq":1,"id":7,"x":-400,"y":30,"duration_ms":4000}"#).is_ok());
        for value in [
            r#"{"v":2,"seq":1,"id":7,"x":0,"y":0,"duration_ms":4000}"#,
            r#"{"v":1,"seq":0,"id":7,"x":0,"y":0,"duration_ms":4000}"#,
            r#"{"v":1,"seq":1,"id":128,"x":0,"y":0,"duration_ms":4000}"#,
            r#"{"v":1,"seq":1,"id":7,"x":-2147483648,"y":0,"duration_ms":4000}"#,
            r#"{"v":1,"seq":1,"id":7,"x":0,"y":0,"duration_ms":999}"#,
            r#"{"v":1,"seq":1,"id":7,"x":0,"y":0,"duration_ms":4000,"html":"bad"}"#,
        ] {
            assert!(parse(value.as_bytes()).is_err());
        }
        assert!(parse(&vec![b' '; 4097]).is_err());
        let hidden =
            parse(br#"{"v":1,"seq":2,"id":7,"x":0,"y":0,"duration_ms":1000,"visible":false}"#)
                .unwrap();
        assert!(!hidden.visible);
    }
}
