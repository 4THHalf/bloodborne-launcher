# Launcher notification overlay

## Investigation (2026-10-13)

Recommendation, revised after the maintainer’s process suggestion: the launcher repository
owns a small Tauri/React overlay helper executable. The runtime starts it and owns its session
and event delivery. The helper owns window lifetime, styling and animation. Start with trophy
notifications. This is a proposed design, not a working
fullscreen overlay. No performance or compatibility measurements have been made.

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

No game runs were needed for this source/documentation review. Fullscreen acceptance, real
unlock display, overhead and Windows/Linux behavior remain untested. Do not enable by default
until the applicable checks pass. N4 remains paused.
