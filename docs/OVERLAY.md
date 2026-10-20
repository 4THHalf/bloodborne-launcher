# Trophy notifications

[← Documentation](README.md)

Trophy notices are shown by a small separate program, `bloodborne-overlay`, built from this
repository (a Tauri window with its own React entry point). The runtime starts it for its game
session and tells it what to show; the launcher itself does not need to be running.

## How it works

- **Who decides.** The runtime accepts an unlock and records it durably first; only newly
  earned, eligible trophies are sent. Duplicates, reloads and edited saves never notify. If the
  helper is slow or gone, the game and the recorded history are unaffected.
- **Delivery.** The runtime starts `bloodborne-overlay --runtime` (`BB_OVERLAY_HELPER`, set by the
  app; `BB_TROPHY_OVERLAY=0`: off) on a worker thread and writes versioned messages to its
  inherited pipe: trophy ids, the title, and the game window's position, size and focus every
  250 ms. One helper per game session; it exits when the pipe closes. No pixels, game memory,
  game files, database or network.
- **The window.** Hidden and unfocused until a notice; no decorations, no Dock entry, mouse
  passes through; shown on every Space, including fullscreen ones. One card at a time from a
  16-entry queue: five seconds (platinum seven), a one-second gap, stale notices dropped after
  30 seconds. Cards use the public catalog's names and grade cups with metallic tints; reduced
  motion is respected.
- **Only over the game.** While a notice is up, every 250 ms, the helper checks that the front
  normal window on screen (layer 0 in `CGWindowListCopyWindowInfo`, window metadata only) belongs
  to the game, its parent process; otherwise the notice hides until the game is in front again.
  SDL's focus and macOS's "frontmost application" both proved unreliable for a game without an
  app bundle. If the helper ever becomes the active app it gives activation straight back
  (`NSApp.deactivate`), so the game keeps its keyboard and controller.

## Settings and cost

Advanced → Launcher → **Trophy notifications**, on by default since 0.7.0. One valid off/on pair
measured 44.7 against 42.8 FPS and 1.3 ms more p90 with the helper. Recording and the launcher's
trophy views work either way.

## Development

- `npm run tauri dev` builds the debug helper first; it loads its page from the dev server
  (port 5173), so without it the helper never becomes ready.
- `BB_TROPHY_OVERLAY_PREVIEW=1` (runtime) sends five labelled previews (four grades and a DLC
  trophy) through the same bridge without recording anything. `tools/test_overlay.py` is a standalone
  synthetic host.
- Game test windows are background-only apps, so focus behaves unlike a launcher session; check
  focus-sensitive changes in a launcher session.

## Platforms

macOS works (fullscreen Spaces, window-order check). Windows needs a non-activating window
tested in windowed and borderless modes. Linux: X11 to test; on Wayland a window cannot place
itself next to another app's, so placement is off there.
