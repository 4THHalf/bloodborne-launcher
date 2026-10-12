# Saves

[← Documentation](../README.md)

The **Saves** tab is available in the development launcher; it is not in v0.5.1.
A profile is a complete set of saves, so it can contain multiple characters.

## Choose a profile

**Original** uses your existing save folder. It is not moved or renamed. A custom save folder
chosen in an older launcher is kept. Choose **Use this save** on another profile to make it
active for the next launch. **Play** starts that profile; it does not automatically select a
particular character inside the game.

Stop the game before switching profiles, copying saves or restoring a backup. The launcher
also checks for a game started outside it.

## Create a profile

Enter a name under **New profile**. Choose **Fresh save** to start over, or choose an existing
profile under **Start from** to copy its complete save set. Press **Create**, then **Use this
save** on the new profile. Creating a copy does not change which profile is active.

The tab shows the size and file modification time of each save set. Character names, stats,
playtime and progression are not displayed yet.

## Back up and restore

The launcher backs up the active save set before every launch. **Back up** creates another
copy while the game is stopped. Backups include supporting save metadata and save-memory files;
shader caches are shared between profiles and are not copied into backups.

Choose a dated backup and press **Restore**. Confirm to replace that profile's save set. A
backup of its current contents is made first. Restoring does not change the active profile.
An incomplete copy is not added to the list. If a restore is interrupted during replacement,
the previous files are retained and the launcher blocks that profile until it is recovered.

Backups are kept without automatic pruning in this first version. **Backups → Show in Finder**
opens their folder. Keep at least one known-good backup if you remove older copies manually.

## Where files live

Under `~/Library/Application Support/bloodborne_mac/`:

- `user/`: the original user directory, unless you selected a custom one.
- `save-profiles/<id>/user/savedata/`: each named profile's complete save set.
- `save-backups/<id>/<backup-id>/savedata/`: backup copies; the original profile's id is `original`.
- `launcher.json`: the active profile selection and launcher settings.

Profiles and backups require regular folders and files; linked or special files are rejected.
Import/export and character editing are planned for later steps.
