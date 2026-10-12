# Profiles and saves

## Roadmap (2026-10-12)

Completed stages: **1/4 (25%)** `█████░░░░░░░░░░░░░░░`
This counts completed stages, not estimated development time.

| Stage | Scope | Status |
|---|---|---|
| S0 | Profile ownership, first-profile setup, complete save sets, active selection, backup and restore | Implemented; SQLite, native tests and browser checks |
| S1 | Read-only character preview: verified names, level, attributes, playtime and progression | Planned |
| S2 | Import/export and backup history management | Planned |
| S3 | Editor for verified fields, copied-save validation, runtime trophy capture and profile trophy view | Planned |

## Ownership and first setup

New installations and upgrades without ownership records show a required first-profile window.
No Play action is available until setup completes; `--play` is also refused by the backend.
The entered name owns the existing original user directory without moving or rewriting its save
files. Existing development save-profile directories are adopted as identities, and a previously
selected development profile remains active. Custom original save paths and old backup ids are
retained. Setup cannot be repeated to overwrite established ownership. A damaged database reports
an error; it is not treated as a fresh installation.

The Profiles tab creates, copies, selects and renames identities. Each has exactly one complete
save set, which can contain multiple characters. The Saves tab shows the active identity only.
The active profile is also visible next to Play. Names are trimmed and normalized with Unicode lowercase for duplicate checks;
ids and save paths remain unchanged on rename. Copying a save creates another identity with
separate saves and backups; trophy records are not copied. Profile deletion is not implemented.

## SQLite and Drizzle

`launcher.db` lives in the existing launcher data directory (`bloodborne_mac` on macOS).
The schema is in `src/db/schema.ts`; `npm run db:generate` generates checked-in SQL and snapshots.
Rust applies those exact migration files, with each migration and its history entry in one
transaction. A test checks that the native migration list matches Drizzle's journal.

| Table | Ownership |
|---|---|
| `profiles` | Stable identity, display name and normalized unique name |
| `save_sets` | Required profile foreign key, unique profile and user directory |
| `launcher_state` | One active-profile record and the original/shared-cache directory |
| `backups` | Required save-set foreign key, published path, timestamp and size |
| `trophy_unlocks` | Unique profile/title/trophy key and timestamp; collection is not implemented |
| `pending_operations` | File-operation journal with pending, complete, failed or interrupted status |

The structure follows Leari: Drizzle schema and typed service reads, TanStack Query hooks and UI;
SQLx in Rust opens the same SQLite file. Native connections use WAL, foreign keys and a busy
timeout. The Tauri SQL plugin is loaded at the absolute database path returned by Rust. The
adapter exposes generated SELECT queries; SQL and file mutations go through native commands.
The plugin's generic SELECT endpoint is not itself a read-only security boundary. The frontend
has no execute permission and app services use only the constrained adapter.

SQLite records metadata, not game save contents. `bbport.ini` and launcher startup settings keep
their existing formats for runtime compatibility. The active profile in SQLite is authoritative;
the legacy `save_profile` setting is used only for the initial development-profile adoption.

## Save files and recovery

Keep the original user folder in place. Named directories remain under
`save-profiles/<id>/user/`, with profile metadata retained as a recovery manifest. Published
manifests are reconciled into SQLite, including a completed create interrupted before indexing.
Missing registered profile folders or mismatched paths block use instead of selecting another
save. On startup, unfinished journal entries are marked interrupted after inventory reconciliation.

Set `BB_USER_DIR` to the active profile, `BB_USER_NAME` to its name, and `BB_GPU_USER_DIR` to the
original directory for shared shader/pipeline caches. These values override inherited and extra
environment values. Guest names are capped at 15 UTF-8 bytes to fit 16-byte caller buffers; the launcher keeps the
full display name. The runtime still presents guest user id 1; isolation is by launch directory.
There is no switching during gameplay.

Copy the entire `savedata/` subtree, including metadata and save-memory files. Publish staged
copies only after completion. Create a recovery backup before every launch and restore. Restore
prepares its replacement beside the live directory, retains the previous directory, and rolls
back if replacement fails. An interrupted restore with retained previous files blocks profile
use and reports its recovery path. Database transactions do not make filesystem copies atomic;
this is not a power-loss durability guarantee.

Native commands use the game mutex and `pgrep -x bb-probe` to reject changes while a game runs.
The launcher is a single instance, serializing its file operations. An externally started game
at exactly the same time remains outside this launcher contract. Backups are not automatically
pruned yet. Restoring a save leaves profile identity and trophy history intact.

## Validation and boundaries

Fourteen native tests use temporary directories with synthetic data only. They cover migration rollback,
foreign keys, missing/corrupt storage, atomic adoption, completed-copy recovery, name uniqueness,
complete-set copying, pre-restore copies, interrupted restore, unsafe paths, active-save launch
routing, and preservation of trophy history on restore. Browser checks use the in-memory preview
backend for both fresh and upgrade setup, mandatory onboarding, identity operations and contextual
save management. They do not exercise the real game or capture screenshots.

Character parsing, editing and actual trophy collection remain future work. Game-derived files
and analysis are never committed. Tests never write the player's save or settings.
