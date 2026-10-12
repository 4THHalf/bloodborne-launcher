# bloodborne-launcher

The launcher for bloodborne-native. Tauri (Rust), React and TypeScript, with 20 languages.
macOS packaging and runtime integration work today; Windows and Linux game integration are
planned. Keep profile/save logic independent of the platform and renderer.

## Layout and checks

- `src/`: frontend. Follow `README.md` conventions: one component per file, `@/` imports,
  folder exports, TanStack Query for backend data, Zustand for local state, all text in i18n.
- `src-tauri/src/`: Rust backend; synthetic save tests use temporary folders, never player data.
- `docs/`: profile/save design and roadmap. Keep this repository's wiki in step.
- `npm ci && npm run check && npm run build`.
- `cd src-tauri && cargo test && cargo clippy -- -D warnings`.
- Standalone development: set `BB_REPO` to a bloodborne-runtime checkout for `npm run tauri dev`.
  Browser preview (`npm run dev`) uses an in-memory backend and does not need the game.

## Repositories and ownership

The launcher is independent, included as `launcher/app/` in bloodborne-mac. That app also owns
`runtime/` (bloodborne-runtime), which owns `recomp/` (bloodborne-recomp). Before editing this
submodule, checkout `main` and pull. Commit and push in the repository owning the files, then
bump the submodule pointer in the parent app and push it. Before every push, run
`git remote get-url origin`. Ask before force-pushing or tagging a release. Commit subjects are
short; bodies explain what changed, why and measured results, with the session's co-author trailer.

## Data and game testing

Never write the player's saves/settings while testing. File tests use synthetic temporary data.
Game tests use bloodborne-runtime's `tools/test_run.sh NAME [VAR=...]` only, which copies saves.
Check `pgrep -x bb-probe` first, one game at a time. Tell the user before batches; stop when asked.
Never capture the user's screen. Only the runtime's `tools/grab.sh` may capture game frames.
No game code, generated/decompiled game code, game files or game-memory records in Git or wikis.
No `rm -rf` on variable shell paths; no `xargs ... sh -c`.

## Credits and docs

Keep bbport and shadPS4 credited in README, docs/CREDITS.md and the wiki. Any adopted third-party
code needs credits in those docs, the wiki, the commit message and a short code comment naming
the source repository and commit. Docs use plain, precise English and measured numbers. Status
progress bars have 20 blocks, one per 5%. Do not claim Windows/Linux game integration or trophy
persistence before they are implemented and tested.
