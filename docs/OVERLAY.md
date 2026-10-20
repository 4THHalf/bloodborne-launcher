# Launcher notification overlay

## Investigation (2026-10-13)

Recommendation, revised after the maintainer’s process suggestion: the launcher repository
owns a small Tauri/React overlay helper executable. The runtime starts it and owns its session
and event delivery. The helper owns window lifetime, styling and animation. Start with trophy
notifications. The first opt-in synthetic helper prototype is implemented and was visually accepted in
windowed and fullscreen tests on this Mac. Runtime spawning, accepted unlock delivery and
SDL geometry are now implemented on development main. The new gameplay gates are recorded
below; Windows/Linux overlay behavior remains unaccepted.

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

Use an inherited local pipe for versioned, bounded messages, with increasing sequence numbers, trophy identity and window state. Each inherited pipe
belongs to one runtime session; no shared/global notification socket is used. The helper needs no game files, SQL access or network.
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
first card. Share theme tokens with the launcher, while keeping the overlay page small. The public
trophy catalog supplies verified names and grades; original requirement summaries use English
fallback where no verified localization exists. No game trophy files, artwork or keys are shipped.

## Platform handling

- macOS: the pinned Tauri 2.12.0 source gates transparent windows behind `macos-private-api`;
  the helper now enables that feature. Its macOS adapter presents the unfocused window in
  fullscreen Spaces without activating the launcher.
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
only. The accepted-event bridge and lifecycle are implemented. Natural gameplay unlocks remain
unobserved, and one valid matched performance pair was slower with the helper. Keep it opt-in;
N4 remains paused.

## Synthetic helper prototype (2026-10-13)

`src-tauri/src/bin/bloodborne-overlay.rs` is a separate executable. It requires `--prototype`,
uses an isolated application identifier and an incognito webview, and does not initialize
profiles, SQLite, tray controls or the main launcher. Its dedicated `overlay.html` entry uses
React and CSS opacity/translation animation, with reduced-motion handling. All test UI is
English. The main launcher remains the default executable.

Input is newline-delimited JSON with `v`, increasing `seq`, trophy `id`, logical `x`/`y` and
`duration_ms`; optional `visible=false` hides the window. Messages are capped at 4,096 bytes,
queued in an eight-entry channel, and validated for version, id, position and duration.
There is one expiry worker. The runtime now paces accepted notifications sequentially. Closing stdin ends the helper.
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

Frontend checks/build, 44 launcher tests, two helper tests and Clippy for all
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
(`BB_LANGUAGE=1 BB_UI_LANGUAGE=en`). The packaged helper and runtime launch path are described below.

## Accepted unlock bridge (2026-10-13)

The runtime starts the standalone `bloodborne-overlay --runtime` executable when
`BB_OVERLAY_HELPER` names an absolute executable and `BB_TROPHY_OVERLAY` is not zero.
The macOS launcher discovers its sibling executable when Trophy notifications is enabled;
the setting is on by default since 0.7.0. packaging builds it with
`tauri/custom-protocol`, copies it into Contents/MacOS and signs it. It embeds the production
React assets and needs no Vite server. Standalone runtime launches may set the path explicitly.

One host worker owns spawn, readiness and pipes. Accepted unlocks are queued only after the
append and fsync succeed, with startup history used to suppress duplicates. A 16-entry queue
paces cards five/seven seconds apart with a one-second gap; stale notifications expire after
30 seconds. Full queues affect only display. Startup has a 20-second readiness budget;
broken pipes cannot signal SIGPIPE on guest threads. The worker holds no guest or renderer
lock while creating the UI process. The helper receives IDs/title and window metadata only;
it needs no profile database, game dump, key, network or save access. EOF ends its owned session.

The SDL event thread reports geometry/focus every 250 ms. The core bridge only copies those
values under its own mutex; a host worker sends pipe messages. Moving, fullscreen switching,
minimizing and losing focus update placement/visibility without restarting a card’s lifetime.
Positions use SDL desktop logical coordinates on macOS. Wayland placement is disabled;
Windows/Linux coordinate/focus adapters still require verification before normal integration.
No screen pixels or game memory are captured.

The public catalog keeps ID, grade and base/DLC group independent of each trophy’s locale
translations. Verified names cover en, de, it, es-ES, es-419, ru, pl, nl, fi, nb, tr, zh-CN,
zh-TW, pt-BR and fr. The Latin American Spanish UI uses the public es-MX list; nb uses the
public Norwegian list. English, pt-BR and fr have original short requirement summaries.
Missing names/descriptions fall back independently to English. Grade labels and notification
headings follow the launcher’s 20 UI locales. All acceptance tests use English.
The catalog’s 40 names per listed locale were checked against the trophy rows by ID; an
earlier import that skipped rows and included page navigation labels was corrected.

Bronze, silver, gold and platinum use one lucide cup emblem with original metallic gradients and grade labels. Platinum uses seven
seconds; other grades use five. The launcher’s trophy details separate 34 base-game trophies
from six Old Hunters trophies. No game icons, extracted metadata or decryption keys are shipped.
The local runtime awards platinum only after all 33 other base-game IDs are recorded.
Notification backgrounds carry the matching metal tint at 35–45% opacity, with a text shadow
for contrast over gameplay.

`BB_TROPHY_OVERLAY_PREVIEW=1` starts the helper in clearly labelled preview mode and schedules
five public-ID previews (four grades, plus a DLC trophy) through the same SDL/pipe bridge.
It does not call the trophy service or append test unlocks. `tools/test_overlay.py` remains a
standalone synthetic visual host. A fake helper in normal runtime tests verifies readiness,
geometry and delivery after durable unlocks, with duplicate and edited-save suppression.
Catalog tests check counts, grouping, sources, localized names and English fallback.
Game-preview checks do not establish that a trophy was earned naturally during gameplay.

Development `npm run tauri dev` builds the matching debug helper before starting Vite and the
launcher. Browser-only `npm run dev` is unchanged. The helper’s Tauri context includes only its
own window and two narrow readiness/language commands, with event-listener permissions;
profile, database and launcher commands are not registered. No new database migration is needed.

## Live bridge checks (2026-10-13)

Four English, untraced, isolated copied-save game runs alternated notifications off/on.
The first windowed pair had stable draw counts near 1,268 per frame and is the valid
throughput comparison. The second pair checked fullscreen appearance; the off run had
frontend development activity during its hold, and the on run changed from 823 to 1,340
draws per frame, so neither establishes a performance gain.

| Run | Mode | FPS | p90 ms | Game CPU % | Stage-B us/draw | Result |
|---|---|---:|---:|---:|---:|---|
| `trophy_live_off1` | Windowed, off | 44.7 | 25.3 | 498 | 4.83 | No fault or assertion |
| `trophy_live_on1` | Windowed, on | 42.8 | 26.6 | 491 | 5.08 | Five previews sent; no fault or assertion |
| `trophy_live_off2` | Fullscreen, off | 45.8 | 24.6 | 500 | 4.81 | Development activity during hold; excluded |
| `trophy_live_on2` | Fullscreen, on | 50.7 | 22.9 | 491 | 5.27 | Variable draw count; visual check only |

The valid pair lost 1.9 FPS (4.3%) and added 1.3 ms to p90 with the helper. This is
one comparison, not proof of a persistent cost. Notifications default off; local trophy
recording and the launcher view stay active. The maintainer saw all five fullscreen cups,
names, descriptions and the DLC label. The later grade-tinted, more translucent CSS change
passed the frontend build and was accepted by the maintainer in a standalone desktop
preview. It has not had another game-window visual check. No natural
unlock was observed; the previews did not alter trophy history.

A 25-second standalone desktop preview of the release helper showed three English cards
and exited cleanly. Its main process was reported as frontmost in 23 of 24 metadata
samples, unlike earlier game-target prototypes; keyboard-focus behavior was not verified.
A further isolated copied-save run, `trophy_focus_final`, completed at 46.7 FPS and p90
23.2 ms without a fault. The chat app stayed frontmost in all 40 window samples, so the
helper correctly sent no previews. This run is inconclusive for game-target focus and is
not a performance pair. The earlier fullscreen visual report remains the game-target
appearance check. Keep notifications optional (on by default since 0.7.0) while focus behavior and
overhead are investigated.

A second isolated hold, `trophy_focus_active`, completed at 46.9 FPS and p90 22.5 ms
without a fault. Two previews were sent late in the run. A metadata sampler reported
VS Code frontmost in all 36 samples, despite an attempted game activation, and never
reported the helper frontmost. The game-focus result remains inconclusive; this is
not another off/on performance comparison.

The repeated 35-second English desktop preview sent all five cards and exited cleanly.
The maintainer reported that the revised transparency and material colors looked right.
The helper was reported frontmost in 14 of 33 metadata samples; input-focus behavior
was not verified in this desktop test.

### The front-window rule (2026-10-20)

On a second display, with the game fullscreen in its own Space, notices showed over other apps:
SDL kept reporting input focus for the game while another app was in front, so the runtime kept
sending them. macOS's frontmost application cannot decide it either: the game, a process without
an app bundle, was never reported frontmost (VS Code stayed so while the game had the keyboard).
The helper now checks the window order itself: while a notice is up, every 250 ms, the front
normal window on screen (layer 0 in CGWindowListCopyWindowInfo, window metadata only) must belong
to the game, its parent process; otherwise the notice hides, and it comes back while it lasts when
the game is in front again. The decision is logged when it changes. With five previews and the
game fullscreen on the second display, notices showed with the game in front and hid with VS Code
in front. The runtime still waits for SDL's focus before sending, which delayed the first notice;
dropping that wait on macOS is the next step.

**Activation (2026-10-20).** In a fullscreen session on Metal, a real trophy notice took the
keyboard and the controller from the game (SDL ignores the pad in the background) although the
window order was unchanged: the helper had become the active app. Whenever the helper finds itself
active (when it shows a notice and at each 250 ms check), it now gives activation back
(`NSApp.deactivate`) and logs "the helper became the active app". In the following fullscreen test
with five previews the controller kept working; activation did not happen in that run, so the
guard has not been seen acting yet.
