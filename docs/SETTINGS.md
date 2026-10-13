# Settings locations

The launcher groups settings by what the player controls. Display and image options belong in
Graphics; game language, startup behaviour and the active save-folder location belong in Game.
Troubleshooting, experimental runtime controls and debug tools belong in Advanced.

| Tab | Contents |
|---|---|
| Home | Active profile and trophy development status, profile/save shortcuts, game installation folder, edition/readiness and shortcuts to common settings |
| Graphics | Output resolution, monitor, fullscreen, presentation, frame-rate mode/limit and FPS counter; upscaling, sharpening, motion vectors, model detail and all visual effects |
| Controls | Controller selection and controller/keyboard bindings |
| Game | Game language, skipping intro videos and the active profile's save-folder location |
| Profiles | Create, select, rename and delete identities |
| Saves | Active save information, backup/restore and backup-folder location |
| Mods | Mod loading, load order and mod-folder location |
| Patches | Third-party patches and their folder |
| Advanced | Launcher language and hiding during gameplay; new-version checks and release links; experimental draw pipeline, readbacks, preupload and live resolution; free camera/debug menu, logs, profiling, extra environment variables and launcher data-folder location |
| Log | Game output |

The Home frame-rate shortcut opens Graphics. Save folders remain owned by profiles: the Game
folder row reveals the current location, rather than reassigning a profile's storage. Saves
retains its convenient folder-open button next to the active save details.

Moving a control does not change its stored setting key, default or runtime behaviour. Existing
translations move with their controls in all 20 locales. Experimental live-resolution controls
stay in Advanced because they are used for troubleshooting the renderer; ordinary output
resolution remains in Graphics.

See [New-version notifications](UPDATES.md) for automatic and manual release checks.
