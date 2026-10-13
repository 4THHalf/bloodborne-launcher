# Launcher notification overlay

## Investigation (2026-10-13)

Recommendation, revised after the maintainer’s process suggestion: the launcher repository
owns a small Tauri/React overlay helper executable. The runtime starts it and owns its session
and event delivery. The helper owns window lifetime, styling and animation. Start with trophy
notifications. The first opt-in synthetic helper prototype is implemented and was visually accepted in
windowed and fullscreen tests on this Mac. Runtime spawning and real unlock delivery are
still planned; Windows/Linux behavior and production overhead are not accepted.

The runtime already has an ImGui overlay drawn into the Vulkan swapchain by
`BbOverlay::Render`, after the game frame. It currently covers settings, text entry and FPS.
It can support an in-frame notification without another browser, but the maintainer prefers
launcher ownership and web animation. Keep this existing overlay available; this proposal
adds no dependency to the renderer and does not resume N4.

## Options

| Approach | Fit | Limits |
|---|---|---|
| Runtime-started Tauri/React helper, maintained in the launcher repo | Recommended prototype. Shares source assets, locale and styling with the launcher; independent of the GPU backend and main launcher process. | Separate-window fullscreen, focus and placement need platform adapters and measured acceptance. Adds a UI process and webview whose overhead needs measurement. |
| Runtime ImGui notification | Existing path can draw into the game frame, with native animation and no webview dependency. | UI belongs to the runtime; later Metal presentation needs its own ImGui backend. |
| Offscreen browser composited into the game | Could render React directly into the game frame. | Adds browser hosting, texture import/copy and lifetime integration. CEF offers paint callbacks, but this is a larger renderer task than a notification toast. Defer. |

[Tauri's window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/) provides
focusability, cursor passthrough and stacking controls. These controls are the starting point;
they do not establish fullscreen compatibility with this separate SDL game process.
[CEF's render interface](https://github.com/chromiumembedded/cef/blob/master/include/cef_render_handler.h)
describes CPU paint and shared-texture callbacks; integration would require backend-specific
resource handling. No browser engine or third-party panel implementation has been adopted.

## Ownership and event delivery

The runtime remains authoritative for accepting an unlock and recording the durable event.
It starts one helper for its own game session, using the packaged executable path configured
by the launcher or standalone run script. The main launcher need not be running. Do not launch
another copy of the main onboarding/profile UI to show a notification: use a dedicated helper
mode or executable with separate single-instance identity and minimal initialization.

Use an inherited local pipe for versioned, bounded messages, including a session identifier,
trophy identity and window state. The helper needs no game files, SQL access or network.
The launcher continues importing the durable event log into SQLite independently. Notifications
must not mutate history or compete with the launcher for ownership transactions.

Only newly earned, eligible trophies may notify. Before producing live events, establish known
unlocks from the profile-owned event history. Duplicate calls and historical reloads must not
replay notifications. If history cannot be trusted or a write fails, do not claim a newly earned
persisted trophy. Keep this decision in the runtime, not in a renderer callback or UI animation.
Edited saves generate no accepted trophy events. Online-module mode uses the same local gate;
online access policy remains undecided.

Move helper spawning and pipe writes onto host notification work, away from guest execution
and present threads. Use a bounded queue; an unavailable or slow helper cannot stall gameplay
or undo an earned trophy. Log startup/pipe failures and bound retries. Close the pipe on exit;
the helper exits on EOF and discards stale queued session events. Killing either launcher or
helper must leave the game and durable trophy history usable. The runtime owns its helper,
including direct starts; never terminate an unrelated process by executable name.

Start the prototype helper hidden before gameplay, so the first toast can use a ready page.
Measure its idle CPU/memory and startup delay. Lazy startup is an alternative if those costs
warrant it; no zero-cost or instant-start claim is made.

Window geometry, display scale, visibility and focus are window-system metadata. The runtime
may report them through an opt-in host event bridge from its SDL window thread. Do not inspect
screen pixels or guest memory to place the overlay. Use explicit logical/physical coordinate
units and the game's display, rather than assuming the primary monitor.

## Window and frontend

Use a dedicated local React entry point, separate from normal launcher onboarding, settings,
SQL services and game controls. Deliver typed events to that window. Give it only the minimal
Tauri capabilities for notification receipt and lifecycle acknowledgement; it needs no SQL,
filesystem or game-control permissions. Render trophy text as text, never injected HTML.

The window starts hidden and unfocused, without decorations or a taskbar entry, and ignores
cursor input. Showing it must not activate the launcher or change game input. Its lifetime is
independent of the main window's hide-to-tray setting. Hide it when the game loses focus,
minimizes or exits; cancel stale callbacks on quick exit/relaunch. Use a bounded queue and one
card at a time. Hide the whole window when idle and stop animation timers. Respect reduced
motion and provide a notification setting plus a synthetic preview.

Animate opacity and translation with CSS. A new animation package is unnecessary for this
first card. Share theme tokens with the launcher, while keeping the overlay page small. Until
metadata extraction exists, display localized generic trophy text and its id; do not embed a
game trophy catalog or game icons in any repository.

## Platform handling

- macOS: the pinned Tauri 2.12.0 source gates transparent windows behind `macos-private-api`;
  the launcher currently does not enable that feature. Evaluate a non-activating AppKit panel
  for fullscreen Spaces, without changing the launcher's application activation during a toast.
  Apple's [fullScreenAuxiliary behavior](https://developer.apple.com/documentation/appkit/nswindow/collectionbehavior-swift.struct/fullscreenauxiliary)
  and [tauri-nspanel](https://github.com/ahkohd/tauri-nspanel) are relevant implementation
  references, not proof that our cross-process fullscreen case works. Keep any AppKit code
  inside a macOS adapter. If adopting a package or code, pin and credit it under its license.
- Windows: test a non-activating overlay in windowed and borderless modes. Treat exclusive
  fullscreen as unverified until tested; an always-on-top flag is not a compatibility promise.
- Linux: test X11 separately from Wayland. SDL documents that Wayland toplevels cannot set
  their own positions in its [Wayland notes](https://wiki.libsdl.org/SDL3/README-wayland).
  A generic detached Tauri window therefore cannot be assumed to track another app. Report
  unsupported capabilities honestly; retain trophy history and consider system notifications
  or the existing in-frame path as a later fallback. Do not force X11 or claim all Wayland
  compositors support an overlay.

## Implementation order and acceptance

1. Build an opt-in standalone helper with a synthetic host sending events over a pipe. Verify windowed placement,
   fullscreen Spaces, focus, cursor passthrough and main-window tray behavior on this Mac.
2. Add runtime helper lifecycle, the optional SDL geometry/live-event bridge and session queue. Use synthetic
   tests for duplicate events, edited saves, stale sessions, failures and rapid exit/relaunch.
3. Connect the React card to live unlocks, with notification settings and reduced motion.
4. Run announced isolated game checks, alternating overlay off/on. Compare FPS, frame-time
   p90, stage-B GPU-thread time per draw and game/launcher CPU and memory. Measure both idle
   overlay and visible animation. Record regressions before changing the default.

The initial source review required no game runs. The prototype checks below cover this Mac
only. Real unlock delivery, full lifecycle integration and matched production performance
gates remain open. Keep the helper opt-in; N4 remains paused.

## Synthetic helper prototype (2026-10-13)

`src-tauri/src/bin/bloodborne-overlay.rs` is a separate executable. It requires `--prototype`,
uses an isolated application identifier and an incognito webview, and does not initialize
profiles, SQLite, tray controls or the main launcher. Its dedicated `overlay.html` entry uses
React and CSS opacity/translation animation, with reduced-motion handling. All test UI is
English. The main launcher remains the default executable.

Input is newline-delimited JSON with `v`, increasing `seq`, trophy `id`, logical `x`/`y` and
`duration_ms`; optional `visible=false` hides the window. Messages are capped at 4,096 bytes,
queued in an eight-entry channel, and validated for version, id, position and duration.
There is one expiry worker. This prototype replaces the displayed card with the latest
notice; production sequential trophy delivery is still planned. Closing stdin ends the helper.
Only its event-listen permissions and readiness command are exposed. The helper needs no SQL
or filesystem plugin. It uses the direct objc2 AppKit bindings inside a macOS adapter.

The first visual attempt (`overlay_on1`) sent no previews because VS Code had focus. It is
an idle-helper check, not visual acceptance. The next attempt (`overlay_on2`) found a real
focus defect: Tauri/Tao's macOS `show()` activates the app even when the window is unfocusable.
That helper was deliberately stopped. The corrected adapter calls `NSWindow.orderFrontRegardless`
on the UI thread, preserving foreground ownership. No third-party panel code was copied.

Corrected windowed and fullscreen tests were reported as **perfect** by the maintainer.
The windowed helper sent 11 previews, never became frontmost in 83 samples, and exited with
code 0 on EOF. The fullscreen helper showed three previews, never became frontmost in 82
samples and also exited with code 0. These are sampled checks, not a proof of every focus
transition or input device. The fullscreen geometry probe initially selected a 32-point
companion/titlebar window; filtering small windows prevented that selection but then missed
some fullscreen geometry. Use the proposed SDL window-state bridge for production placement.
The positive visual report does not close that geometry/lifecycle work.

`tools/test_overlay.py` is a synthetic host, not runtime integration. Its macOS Swift utility
reads window bounds and frontmost-process metadata only; it never captures pixels.
`--preview-unfocused` allows observation while using the chat and is a testing override, not
the proposed normal focus policy. No trophy history was modified or real unlock earned.

| Run | Hold | FPS | p90 ms | Game CPU % | Stage-B us/draw | Check |
|---|---:|---:|---:|---:|---:|---|
| `overlay_off1` | 45 s | 42.0 | 27.9 | 471 | 5.06 | No helper |
| `overlay_on1` | 45 s | 47.4 | 22.9 | 516 | 4.65 | Idle helper; no visible previews |
| `overlay_fixed_window` | 120 s | 44.9 | 25.1 | 500 | 4.81 | Corrected windowed previews |
| `overlay_fixed_fullscreen` | 120 s | 43.4 | 26.6 | 488 | 4.89 | Corrected fullscreen previews |

The initial off/on pair alternated, but on was idle. The corrected runs have different hold
lengths, a synthetic metadata host and a debug helper/dev server. Fullscreen also included
English-language/HMR and geometry-probe changes; builds ran during development. These are
smoke measurements, not a valid matched overhead or speed comparison. Do not claim a gain
or use this batch as a default-enable gate. `overlay_on2` is excluded from performance
acceptance because it exposed activation and its helper was stopped. All five game runs had
no faults, GPU assertions or invalid-mutex panics; historical assertion counts are unchanged.

The corrected windowed helper's main process averaged 0.16% sampled CPU and reached 75.4 MiB
RSS. The fullscreen helper averaged 0.39% and reached 75.6 MiB. These exclude WebKit processes,
the test host and compositor cost; they are not total overlay resource use. An initial desktop
preview reached 122.9 MiB main-process RSS. Debug/test conditions differ.

Frontend checks/build, all 43 launcher tests, the helper's protocol test and Clippy for all
targets pass locally on macOS. Linux frontend CI and macOS native CI will check the push;
Windows/Linux native helper behavior is untested.

### Reproduce the prototype

Run `npm run dev` for the debug helper's local page, then:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --bin bloodborne-overlay
swiftc tools/overlay_window_macos.swift -o src-tauri/target/debug/overlay-window-macos
python3 tools/test_overlay.py --helper src-tauri/target/debug/bloodborne-overlay \
  --geometry src-tauri/target/debug/overlay-window-macos --seconds 20
```

This opens test previews. For a game target, start the game only with the runtime's
`tools/test_run.sh`, checking `pgrep -x bb-probe` first and announcing the batch. Supply that
owned game's PID with `--pid`; the host does not start or stop the game. Use English
(`BB_LANGUAGE=1 BB_UI_LANGUAGE=en`). The packaged helper and runtime launch path are not yet wired.
