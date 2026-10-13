# Profiles and saves

Profile and save management is included in 0.6.0. Local trophy tracking is on development
main after that release; 0.6.0 still shows the trophy development badge.

## Your first profile

On a new installation or the first launch after updating, the launcher asks you to create a
profile. Enter a name and press **Create profile**. Your existing saves stay in their current
folder and become owned by this profile. Existing development profiles are kept too.

The setup window is required before Play becomes available. If there are no saves yet, the
profile starts empty. You can quit the launcher and finish setup later.

## Profiles

Home shows the active profile prominently below Play, alongside the recorded trophy count and **View trophies**.
Its Profiles and Saves buttons open identity management and the active save's backups.
Changing the active profile updates this card.

Open **Profiles** and press **New profile**. Enter a name in the modal, then press **Create**.
Every new profile starts with empty saves and backups. Existing saves are adopted only during
first-profile setup when upgrading from a launcher without profile ownership. Each profile has
its own complete save set, which can contain multiple characters. Creating a profile does not
change which one is active. Choose **Use profile**, or use the **Active profile** selector above Play.

Renaming a profile changes its displayed name without moving its files. Stop the game before
changing profiles or managing saves. Each profile shows a recorded trophy count. **View trophies**
opens a modal with unlock dates. Tracking is local and starts with this runtime; previous save
progression is not imported. Development main shows public trophy names, original requirement
summaries and grade cups. The runtime awards local platinum after the other 33 base-game
trophies are recorded.

## Save editor

The Saves tab shows **Save editor — In development**.
It is informational; save editing and character previews are not available yet.

## Delete a profile

Switch to a different profile first, then press **Delete** on the profile you want to remove.
The confirmation names the profile and warns that its saves and backups will be permanently
deleted. Press **Cancel** to keep it, or **Delete** to confirm. The active and last profiles are
protected, and the game must be stopped.

Deleting the original migrated profile preserves the shared shader cache. If some files cannot
be removed, the launcher reports the error and keeps cleanup pending. Resolve the file error
and restart with the game stopped to retry. The deleted profile will not reappear.

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

- `launcher.db`: profile ownership, active selection, backup metadata and trophy records.
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
save ownership and trophy history live there. For an unexpected schema or ownership
error, retain the data folder and report the full message. Quit the launcher before copying the
folder for recovery, including any `launcher.db-wal` and `launcher.db-shm` files. Schema-upgrade
snapshots are kept in `database-backups/`; they are separate from your game-save backups.

## Edited saves and trophies

Edited saves cannot earn new trophies. A marked edited save shows a warning alongside its
profile and trophy count. Previously earned trophies remain. Backups retain the edited marker;
restoring a clean pre-edit backup restores its own eligibility. The editor remains planned and
will warn before applying edits. External edits are not automatically detected. Online access
policy for edited saves has not been decided; this restriction applies to trophies only.

## Trophy details and notifications (development main)

**View trophies** separates Bloodborne from The Old Hunters and shows names, grades, short
requirement summaries and recorded dates. The 33 non-platinum base-game trophies determine
local platinum; DLC trophies do not. Names have verified public translations in 15 locales,
with English fallback for missing translations. Summaries currently support English,
Brazilian Portuguese and French; other locales use English until verified text is available.
No game artwork or extracted trophy files are included.

Enable **Advanced → Launcher → Trophy notifications** to show accepted unlocks above the
focused macOS game. This optional setting defaults off while overhead checks remain open.
All four grades use one cup emblem with different metallic finishes; platinum stays visible
for seven seconds, other grades for five. Notifications hide when the game loses focus or is minimized. An unavailable
helper leaves earned history recorded. Local tracking does not synchronize with PSN.
