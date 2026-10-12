# Profiles and saves

This feature is in the development launcher; v0.5.1 does not include it.

## Your first profile

On a new installation or the first launch after updating, the launcher asks you to create a
profile. Enter a name and press **Create profile**. Your existing saves stay in their current
folder and become owned by this profile. Existing development profiles are kept too.

The setup window is required before Play becomes available. If there are no saves yet, the
profile starts empty. You can quit the launcher and finish setup later.

## Profiles

Open **Profiles** and press **New profile**. Enter a name in the modal, then press **Create**.
Every new profile starts with empty saves and backups. Existing saves are adopted only during
first-profile setup when upgrading from a launcher without profile ownership. Each profile has
its own complete save set, which can contain multiple characters. Creating a profile does not
change which one is active. Choose **Use profile**, or use the **Active profile** selector above Play.

Renaming a profile changes its displayed name without moving its files. Stop the game before
changing profiles or managing saves. Trophy collection is planned; each profile owns its history.

## Saves and backups

The **Saves** tab shows the active profile's save size, modification time and backups. Character
names, stats and progression are not displayed yet.

The active save set is backed up before every launch. **Back up** creates another recovery copy.
Choose a dated backup and press **Restore**, then confirm. The current contents are backed up
first. Restore leaves the profile identity and any stored trophy history intact.

Backups include supporting save metadata and save-memory files. Shader caches stay shared, so
creating a profile does not discard compiled shaders. **Show in Finder** opens the save or
backup folder. Backups are retained without automatic pruning in this version.

If a restore is interrupted during replacement, previous files are retained and the launcher
blocks that profile until recovery. Invalid storage reports an error rather than silently
starting a different save.

## Storage

Under `~/Library/Application Support/bloodborne_mac/`:

- `launcher.db`: profile ownership, active selection, backup metadata and future trophy records.
- `user/`: original saves, unless you had configured a custom directory.
- `save-profiles/<id>/user/savedata/`: each additional profile's save set.
- `save-backups/<id>/<backup-id>/savedata/`: complete recovery copies.

The original directory, existing backup ids and custom save paths are preserved during setup.
Profiles and saves must be regular folders/files; links and special files are rejected.
Import/export and character editing are planned.

## If profiles cannot load

The launcher shows the error with **Retry** and **Show in Finder**. Resolve disk-space,
permissions or another running launcher's lock, then retry. If the error says the database is
from a newer launcher, install the latest version. The fixed launcher automatically recognizes
the initial migration's former name; it does not recreate existing tables.

Do not delete `launcher.db` or create replacement profiles to bypass an error. Your identity,
save ownership and future trophy history live there. For an unexpected schema or ownership
error, retain the data folder and report the full message. Quit the launcher before copying the
folder for recovery, including any `launcher.db-wal` and `launcher.db-shm` files. Schema-upgrade
snapshots are kept in `database-backups/`; they are separate from your game-save backups.
