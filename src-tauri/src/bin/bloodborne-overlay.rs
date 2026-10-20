// SPDX-License-Identifier: GPL-2.0-or-later
//! Standalone notification helper. No profile database, game files or launcher startup.
#[path = "../trophy_catalog.rs"]
mod trophy_catalog;
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
    #[serde(default)]
    kind: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(skip_deserializing)]
    translations: std::collections::BTreeMap<String, trophy_catalog::Text>,
    #[serde(skip_deserializing)]
    name: Option<String>,
    #[serde(skip_deserializing)]
    description: Option<String>,
    #[serde(skip_deserializing)]
    grade: trophy_catalog::Grade,
    #[serde(skip_deserializing)]
    group: Option<String>,
    #[serde(skip_deserializing)]
    preview: bool,
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
    let mut n: Notice = serde_json::from_slice(line).map_err(|e| e.to_string())?;
    if n.v != 1
        || n.seq == 0
        || !matches!(n.kind.as_str(), "" | "window")
        || n.title.as_ref().is_some_and(|s| {
            s.len() > 16
                || s.is_empty()
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        })
        || n.id >= 128
        || n.x.abs_diff(0) > 100_000
        || n.y.abs_diff(0) > 100_000
        || !(1000..=10_000).contains(&n.duration_ms)
    {
        return Err("invalid notification fields".into());
    }
    if let Some(trophy) = n
        .title
        .as_deref()
        .and_then(|title| trophy_catalog::find(title, n.id))
    {
        n.translations = trophy.translations.clone();
        n.name = Some(trophy.text("en").name.clone());
        n.description = trophy.text("en").description.clone();
        n.grade = trophy.grade;
        n.group = Some(trophy.group.clone());
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
#[tauri::command]
fn overlay_language() -> String {
    std::env::var("BB_UI_LANGUAGE").unwrap_or_else(|_| "en".into())
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

/// The game's window (this helper's parent process) is the front window on screen. A fullscreen
/// game on its own Space kept SDL's input focus while another app was in front, so the notice, on
/// every Space above normal windows, showed over that app; and macOS does not count the game, a
/// process without an app bundle, as the frontmost application even while it has the keyboard.
/// So the window order decides: the first normal window on screen (layer 0, front to back).
/// Window metadata only; nothing on screen is captured.
#[cfg(target_os = "macos")]
fn game_in_front() -> bool {
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use core_graphics::window::{
        copy_window_info, kCGNullWindowID, kCGWindowLayer, kCGWindowListExcludeDesktopElements,
        kCGWindowListOptionOnScreenOnly, kCGWindowOwnerPID,
    };
    let game = std::os::unix::process::parent_id() as i64;
    let Some(windows) =
        copy_window_info(kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements, kCGNullWindowID)
    else {
        return true;
    };
    // The keys are CoreGraphics' constant strings.
    let (layer_key, pid_key) =
        unsafe { (CFString::wrap_under_get_rule(kCGWindowLayer), CFString::wrap_under_get_rule(kCGWindowOwnerPID)) };
    let number = |dict: &CFDictionary<CFString, CFType>, key: &CFString| {
        dict.find(key).and_then(|v| v.downcast::<CFNumber>()).and_then(|n| n.to_i64())
    };
    let front = windows.iter().find_map(|item| {
        // Each item of the window list is a dictionary, owned by the list.
        let dict: CFDictionary<CFString, CFType> =
            unsafe { CFDictionary::wrap_under_get_rule(*item as CFDictionaryRef) };
        (number(&dict, &layer_key) == Some(0)).then(|| number(&dict, &pid_key)).flatten()
    });
    let in_front = front.is_none_or(|pid| pid == game);
    // Logged when it changes (the game's log): which process owned the front window.
    static LAST: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    let pid = front.unwrap_or(-1);
    if LAST.swap(pid, Ordering::Relaxed) != pid {
        eprintln!("Overlay: front window pid {pid}, game pid {game}: notices {}", if in_front { "shown" } else { "hidden" });
    }
    in_front
}
#[cfg(not(target_os = "macos"))]
fn game_in_front() -> bool {
    true
}

/// Shows the notice window while the game is in front, hides it otherwise (main thread).
fn follow_game(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    if game_in_front() {
        if !window.is_visible()? {
            show_without_focus(window)?;
        }
        Ok(())
    } else {
        window.hide()
    }
}

fn main() {
    let prototype = std::env::args().any(|a| a == "--prototype");
    if !prototype && !std::env::args().any(|a| a == "--runtime") {
        eprintln!("This helper requires --runtime or --prototype and JSON lines on stdin.");
        std::process::exit(2);
    }
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    context.config_mut().identifier = "io.github.bloodborne-native.overlay".into();
    let generation = Arc::new(AtomicU64::new(0));
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![overlay_ready, overlay_language])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("overlay.html".into()),
            )
            .title("Bloodborne notifications")
            .inner_size(460.0, 220.0)
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
                    let mut notice = match parse(&line) {
                        Ok(n) if n.seq > last => n,
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Overlay input: {e}");
                            continue;
                        }
                    };
                    notice.preview = prototype;
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
                let mut active = false;
                // While a notice is up, it follows the game: hidden whenever another app is in
                // front, back while it lasts when the game is.
                const FOCUS_CHECK: Duration = Duration::from_millis(250);
                loop {
                    let received = match deadline {
                        Some(at) => receiver.recv_timeout(
                            at.saturating_duration_since(std::time::Instant::now()).min(FOCUS_CHECK),
                        ),
                        None => receiver
                            .recv()
                            .map_err(|_| std::sync::mpsc::RecvTimeoutError::Disconnected),
                    };
                    if matches!(received, Err(std::sync::mpsc::RecvTimeoutError::Timeout))
                        && deadline.is_some_and(|at| std::time::Instant::now() < at)
                    {
                        if active {
                            let app = handle.clone();
                            let _ = handle.run_on_main_thread(move || {
                                if let Some(window) = app.get_webview_window("overlay") {
                                    let _ = follow_game(&window);
                                }
                            });
                        }
                        continue;
                    }
                    match received {
                        Ok(notice) => {
                            if notice.kind == "window" {
                                let app = handle.clone();
                                let showing = active && notice.visible;
                                let _ = handle.run_on_main_thread(move || {
                                    if let Some(window) = app.get_webview_window("overlay") {
                                        let _ = window.set_position(tauri::LogicalPosition::new(notice.x as f64, notice.y as f64));
                                        if showing {
                                            let _ = follow_game(&window);
                                        } else {
                                            let _ = window.hide();
                                        }
                                    }
                                });
                                continue;
                            }
                            active = notice.visible;
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
                                            .set_position(tauri::LogicalPosition::new(notice.x as f64, notice.y as f64))
                                            .and_then(|()| {
                                                window.emit("trophy-notification", &notice)
                                            })
                                            .and_then(|()| follow_game(&window));
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
                            active = false;
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
        let dlc =
            parse(br#"{"v":1,"seq":1,"id":35,"title":"CUSA03173","x":0,"y":0,"duration_ms":5000}"#)
                .unwrap();
        assert_eq!(dlc.grade, trophy_catalog::Grade::Silver);
        assert_eq!(dlc.group.as_deref(), Some("old_hunters"));
        assert!(dlc.description.is_some());
        let hidden =
            parse(br#"{"v":1,"seq":2,"id":7,"x":0,"y":0,"duration_ms":1000,"visible":false}"#)
                .unwrap();
        assert!(!hidden.visible);
    }
}
