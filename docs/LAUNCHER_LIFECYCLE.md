# Launcher window and game lifecycle

**Advanced → Launcher → Hide launcher during gameplay** is enabled by default. Turn it off
to keep the launcher and Log tab visible while playing. The preference is saved in
`launcher.json` as `hide_launcher`; existing installations keep the default without a database
migration. Changing it during gameplay immediately hides or restores the window.

When a game process starts successfully, the launcher makes its menu-bar/system-tray icon
visible. With automatic hiding enabled, it hides its window. When hidden on macOS, it switches
from Regular to Accessory activation policy, removing the launcher's Dock entry. Windows/Linux use Tauri's taskbar
visibility API; game integration on those platforms remains planned.

The menu contains **Show launcher** and **Stop game**, translated with the launcher language.
Show launcher restores the window without stopping the game. Closing this reopened window
while a game runs returns it to the tray. Closing the launcher while idle keeps the normal quit
behaviour. Stop game uses the launcher's existing process-stop command.

When the owned process exits, including nonzero exits or termination by a signal, the launcher
restores its Dock/taskbar entry, shows and focuses its window, and hides the tray icon. An early
spawn failure does not hide it. Output logging and game-state events continue while hidden.
A process-wait error restores the window and keeps the process handle and Stop action available
while monitoring retries; it is not treated as a confirmed exit.

Visibility changes run on the UI thread and read the current process state. A rapid exit/restart
invalidates the last presented state, so the new launch hides again. Manual reopening cancels
pending automatic hiding for the current run. If tray setup or hiding fails, the launcher stays
available; if restoration fails, the tray remains available for another Show launcher attempt.

## Icon and reference

macOS uses the project's original hunter's sigil as a template icon, tinted by the system for
light/dark menu bars. `src-tauri/icons/tray-icon.svg` is the source; `tray-icon.png` is its 64 px
render. This is original project artwork, not an extracted game asset. Windows/Linux retain the
app icon. Regenerate the template with the Tauri icon CLI into a temporary directory, then copy
its `64x64.png` to `src-tauri/icons/tray-icon.png`.

Window restoration and Dock/taskbar handling follow
[leari-mail/leari 5876482](https://github.com/leari-mail/leari/tree/5876482/src-tauri/src)
(MIT, Pablo Souza). Credit and the original MIT notice are in `docs/CREDITS.md` and
`docs/licenses/leari-MIT.txt`. The launcher uses the public Tauri tray/menu APIs.

## Verification

`cargo test --manifest-path src-tauri/Cargo.toml` includes process polling tests for successful,
nonzero and killed processes, a missing handle and a retryable wait error. There are 37 native
tests in total; profile/save tests continue to use synthetic temporary data.

For a native GUI smoke check on a logged-in desktop:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --example tray_lifecycle
```

Tell the user before running it: it opens a small test window and tray icon. The harness has its
own application identifier, loads only `about:blank`, and runs short-lived synthetic processes.
It does not launch the game, open the profile database, write saves/settings or capture screens.
It verifies native hiding/restoration, disabled automatic hiding, live preference changes,
explicit close with automatic hiding disabled, manual reopen/close, normal/nonzero/killed exits
and rapid restart after reopening. The macOS check passed on 2026-10-12. Windows/Linux GUI
behaviour has not been exercised here. Actual game loading is tested separately with runtime `tools/test_run.sh`.
