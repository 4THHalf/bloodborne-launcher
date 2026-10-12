# Launcher save manager

## Roadmap (2026-10-12)

Completed stages: **1/4 (25%)** `█████░░░░░░░░░░░░░░░`
This counts completed stages, not estimated development time.

| Stage | Scope | Status |
|---|---|---|
| S0 | Saves tab, named complete-save profiles, active selection, fresh/copy profiles, backup and restore | Implemented; synthetic file tests and browser interaction checks |
| S1 | Read-only character preview: verified names, level, attributes, playtime and progression fields | Planned; no parser yet |
| S2 | Import/export with complete-set validation and backup history management | Planned |
| S3 | Editor for verified fields, recovery copies and copied-save game validation | Planned |

## Storage and launch contract

Keep the original user folder where it is. `user_dir` retains its existing meaning; the new
`save_profile` setting defaults to `original`. Named profiles live under
`save-profiles/<id>/user/`. Select the active user directory through `BB_USER_DIR` at launch.
Keep `BB_GPU_USER_DIR` pointing to the original user directory for shared shader/pipeline
caches. The chosen save and cache paths override inherited or extra environment values.
A missing or invalid selected profile blocks launch rather than falling back to another save.

Copy the complete `savedata/` subtree, including metadata and save-memory directories. Do not
interpret character files in S0. Copy into a hidden staging directory and publish with rename
only after the files and profile metadata have been written. Refuse links and special files.
Store profile display names separately from generated directory ids; names never become paths.

Create a backup before each launch and before each restore. Backup failure blocks the action.
For restore, prepare the replacement beside the live directory, retain the previous directory,
and roll back if replacement fails. An interrupted restore with retained previous files blocks
profile use and reports the recovery path. This is not a power-loss durability guarantee.
Backups have no automatic pruning yet. Profile deletion and rename are not implemented in S0.

Serialize save mutations, settings writes and launch with the launcher's game mutex. Reject
mutations while its child is running; also check `pgrep -x bb-probe` for outside runs. The latter
is a check, not an interprocess lock: launching another copy of the game externally at exactly
the same time remains outside this contract. No save changes are applied optimistically in the
UI: wait for backend success and refresh the profile list and configuration.

## Validation

Six Rust tests use temporary directories containing synthetic data only. They verify complete
set copying (including supporting metadata and save-memory files), original preservation, fresh
profiles, pre-restore recovery copies, rejection of path traversal and links, corrupt-backup and interrupted
restore protection, running-child rejection and selected-save/shared-cache launch routing.
No player save or settings were written by these checks; the game was not launched.

The headless browser uses the preview's in-memory backend. It checks creation, duplication,
selection, backups, restore confirmation and disabled controls while running. This verifies the
UI separately from the native file tests, not end-to-end game playback. No screenshots are taken.
TypeScript, ESLint, all 20 locale key/interpolation checks, Rust tests, Clippy and the production
frontend build are required for this slice.

## Later save parsing

First inspect format compatibility on copies, with no game files or save contents committed.
Expose only fields whose offsets, encodings and consistency rules have been verified. Keep a
read-only preview before adding edits. Every edit needs a recovery copy and atomic replacement;
validate loading an edited copy through the isolated runtime test flow. External implementations
must receive the project's required license review and credits before any code is adopted.
