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

The Profiles tab creates, selects and renames identities. Creation uses a button and name modal. Each has exactly one complete
save set, which can contain multiple characters. The Saves tab shows the active identity only.
The active profile is also visible next to Play. Names are trimmed and normalized with Unicode lowercase for duplicate checks;
ids and save paths remain unchanged on rename. Every new profile starts with empty saves,
backups and trophy history; the backend rejects copy requests. Existing saves are adopted only
by the one-time migration from launchers without profile ownership. Inactive profiles can be deleted with a confirmation modal.

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

## Database upgrades and recovery

Migration identities and SQL are immutable once used. Add a new migration for every schema
change; never rename or rewrite an applied migration. Native startup stores the applied SQL and
checks both the migration history and the actual tables, indexes, constraints and foreign keys.
A newer database opened by an older launcher is rejected with an update instruction.

The initial migration was briefly called `0000_giant_rage` before being renamed to
`0000_launcher_storage`. This caused startup to attempt table creation again and report
`table backups already exists`. Startup now recognizes that one known alias only after checking
the complete schema, keeps the original timestamp and records the canonical identity without
rerunning DDL. An exact initial schema with missing history can also be adopted. Partial or
unknown schemas are preserved and reported, rather than guessed at or reset.

Each schema migration and its history entry commit together under `BEGIN IMMEDIATE`. A failed
migration rolls back and can be retried. Before upgrading an existing schema, `VACUUM INTO`
creates a consistent snapshot, including committed WAL contents, under `database-backups/`.
If the snapshot fails, schema changes do not start. Snapshots are retained and are not restored
automatically over newer data. History normalization is transactional and changes no profile,
save, backup or trophy rows.

Startup failures show a recovery screen with Retry, the error details and Show in Finder.
Configuration-loading failures also reach this screen instead of an indefinite loading view.
Do not delete `launcher.db` to recover: it owns profile identity and trophy history. Correct a
transient lock, disk-space or permissions error and retry; unsupported versions require an
updated launcher. Unexpected schema or ownership errors require investigation with the original
data retained. This preserves data on the tested failure paths; it cannot guarantee recovery
from arbitrary disk corruption or hardware failure.

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

## Profile deletion

The profile card's Delete action opens a shadcn `AlertDialog` naming the profile and warning
that its saves and backups will be permanently removed. Initial focus goes to Cancel. The
active profile cannot be deleted: switch to another profile first. The last profile is protected.
All deletion commands use the same game mutex and external-game check as other save operations.
Profile creation also uses the shared shadcn `Dialog` component.

Migration `0001_profile_deletion` adds a nullable `deleted_at` marker without changing earlier
migration SQL. Deletion commits the marker and a pending cleanup operation together before
removing files. Backup and trophy rows are removed, and the name becomes available for reuse.
The profile and save-set rows remain as recovery tombstones; they are excluded from normal
queries and prevent leftover manifests or the original folder from being adopted again.

Remove the named profile's managed directory and its backup directory. For the original
migrated profile, remove only `savedata/` and its backups, preserving the original root's shared
shader/pipeline caches. No other profile's files are removed. Folder ancestors are checked and
symlinked directories are rejected. Filesystem removal is idempotent; it is not part of the
SQLite transaction and cannot be rolled back after confirmation.

An interrupted or failed cleanup remains pending and is retried at startup when no game is
running. Cleanup failure reports the file error and asks for a restart after resolving it;
the deleted profile stays hidden and other profiles remain usable. Successful cleanup marks
the operation complete. Database-upgrade snapshots contain metadata, not copies of game saves.

## Validation and boundaries

Twenty-seven native tests use temporary directories with synthetic data only. They cover migration rollback and retry, renamed/missing migration history, changed SQL,
unsupported newer versions, concurrent startup, WAL-aware upgrade snapshots and failed snapshot
gates, schema mismatch, ownership validation,
foreign keys, missing/corrupt storage, atomic adoption, completed-copy recovery, name uniqueness, fresh-save creation despite existing migrated saves,
complete-set copying, pre-restore copies, interrupted restore, unsafe paths, active-save launch
routing, and preservation of trophy history on restore, deletion of owned saves/backups/trophies, active and
last-profile protection, original-cache preservation, interrupted deletion and unsafe cleanup paths. Browser checks use the in-memory preview
backend for both fresh and upgrade setup, mandatory onboarding, identity operations and contextual
save management, plus persistent configuration/database failures and successful manual retry. The shadcn creation and deletion modals are checked for focus, cancellation, duplicate-name retry,
confirmed deletion and active/running-profile protection. They do not exercise the real game or capture screenshots.

Profile cards show a Trophies row with an In development badge, without unlock counts or actions.
Character parsing, editing and actual trophy collection remain future work. Game-derived files
and analysis are never committed. Tests never write the player's save or settings.
