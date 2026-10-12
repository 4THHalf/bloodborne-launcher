// SPDX-License-Identifier: GPL-2.0-or-later
//! One identity owns each complete save set. First setup binds the existing user folder in
//! place; other identities are the metadata-bearing profile directories created by saves.rs.
use crate::saves;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;
#[derive(Serialize, Deserialize)]
pub struct Registry {
    version: u32,
    pub original_name: String,
    pub original_user_dir: PathBuf,
    pub active: String,
}
#[derive(Serialize)]
pub struct ProfileList {
    pub setup_required: bool,
    pub has_existing_saves: bool,
    pub active: String,
    pub profiles: Vec<saves::Profile>,
}
fn load(data: &Path) -> Result<Option<Registry>> {
    let path = data.join("profiles.json");
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
        Ok(meta) if !meta.is_file() || meta.file_type().is_symlink() => {
            return Err("Invalid profile registry.".into())
        }
        Ok(_) => {}
    }
    let registry: Registry = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if registry.version != 1
        || registry.original_name.trim().is_empty()
        || !registry.original_user_dir.is_absolute()
    {
        return Err("Invalid or unsupported profile registry.".into());
    }
    Ok(Some(registry))
}
pub fn require(data: &Path) -> Result<Registry> {
    load(data)?.ok_or_else(|| "Create your first profile before launching the game.".into())
}
fn write(data: &Path, registry: &Registry) -> Result<()> {
    saves::ensure_directory(data)?;
    let pending = data.join("profiles.json.pending");
    if pending
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err("Invalid profile staging file.".into());
    }
    fs::write(
        &pending,
        serde_json::to_vec_pretty(registry).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(pending, data.join("profiles.json")).map_err(|e| e.to_string())
}
fn owned(data: &Path, registry: &Registry) -> Result<Vec<saves::Profile>> {
    let mut profiles = saves::list(data, &registry.original_user_dir)?;
    profiles[0].name.clone_from(&registry.original_name);
    if !profiles.iter().any(|p| p.id == registry.active) {
        return Err("The active profile is missing. Restore its folder before launching.".into());
    }
    Ok(profiles)
}
pub fn list(data: &Path, original: &Path) -> Result<ProfileList> {
    if let Some(registry) = load(data)? {
        let profiles = owned(data, &registry)?;
        return Ok(ProfileList {
            setup_required: false,
            has_existing_saves: profiles.iter().any(|p| p.bytes > 0),
            active: registry.active,
            profiles,
        });
    }
    // Read metadata only; merely opening the setup window never writes player data.
    let existing = saves::list(data, original)?;
    Ok(ProfileList {
        setup_required: true,
        has_existing_saves: existing.iter().any(|p| p.bytes > 0),
        active: String::new(),
        profiles: Vec::new(),
    })
}
fn name(name: &str, profiles: &[saves::Profile], except: Option<&str>) -> Result<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(
            "Use a profile name between 1 and 80 characters, without control characters.".into(),
        );
    }
    if profiles
        .iter()
        .any(|p| Some(p.id.as_str()) != except && p.name.to_lowercase() == name.to_lowercase())
    {
        return Err("A profile with this name already exists.".into());
    }
    Ok(name.into())
}
pub fn initialize(
    data: &Path,
    original: &Path,
    legacy_active: &str,
    first_name: &str,
) -> Result<()> {
    if load(data)?.is_some() {
        return Err("Profile setup is already complete.".into());
    }
    let existing = saves::list(data, original)?;
    let first_name = name(first_name, &existing, Some("original"))?;
    if !existing.iter().any(|p| p.id == legacy_active) {
        return Err(
            "The previously selected save profile is missing. Restore it before completing setup."
                .into(),
        );
    }
    let original = if original.is_absolute() {
        original.into()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(original)
    };
    write(
        data,
        &Registry {
            version: 1,
            original_name: first_name,
            original_user_dir: original,
            active: legacy_active.into(),
        },
    )
}
pub fn selected(data: &Path) -> Result<saves::Profile> {
    let registry = require(data)?;
    owned(data, &registry)?
        .into_iter()
        .find(|p| p.id == registry.active)
        .ok_or_else(|| "The active profile is missing.".into())
}
pub fn change(
    data: &Path,
    action: &str,
    id: &str,
    new_name: Option<&str>,
    source: Option<&str>,
) -> Result<()> {
    let mut registry = require(data)?;
    let profiles = owned(data, &registry)?;
    match action {
        "select" => {
            if !profiles.iter().any(|p| p.id == id) {
                return Err("Unknown profile.".into());
            }
            registry.active = id.into();
            write(data, &registry)
        }
        "create" => {
            let new_name = name(new_name.unwrap_or_default(), &profiles, None)?;
            if source.is_some_and(|source| !profiles.iter().any(|p| p.id == source)) {
                return Err("Unknown source profile.".into());
            }
            // The directory is its owner's record: a completed create cannot leave an orphan
            // if the launcher exits immediately after publication. Selection stays unchanged.
            saves::create(data, &registry.original_user_dir, &new_name, source)?;
            Ok(())
        }
        "rename" => {
            if !profiles.iter().any(|p| p.id == id) {
                return Err("Unknown profile.".into());
            }
            let new_name = name(new_name.unwrap_or_default(), &profiles, Some(id))?;
            if id == "original" {
                registry.original_name = new_name;
                write(data, &registry)
            } else {
                saves::rename(data, &registry.original_user_dir, id, &new_name)
            }
        }
        _ => Err("Unknown profile operation.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = fs::canonicalize(std::env::temp_dir())
                .unwrap()
                .join(format!(
                    "bb-profile-test-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn original(&self) -> PathBuf {
            self.0.join("custom-user")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn fresh_setup_required_once_then_every_save_has_an_owner() {
        let f = Fixture::new();
        let original = f.original();
        assert!(list(&f.0, &original).unwrap().setup_required);
        assert!(!f.0.join("profiles.json").exists());
        assert!(selected(&f.0).is_err());
        assert!(change(&f.0, "create", "", Some("B"), None).is_err());
        assert!(initialize(&f.0, &original, "original", " ").is_err());
        initialize(&f.0, &original, "original", "Pablo").unwrap();
        assert!(!list(&f.0, &original).unwrap().setup_required);
        assert!(initialize(&f.0, &original, "original", "Overwrite").is_err());
        change(&f.0, "create", "", Some("Second"), None).unwrap();
        let profiles = list(&f.0, &original).unwrap().profiles;
        let second = profiles.iter().find(|p| p.name == "Second").unwrap();
        assert_eq!(selected(&f.0).unwrap().name, "Pablo");
        change(&f.0, "select", &second.id, None, None).unwrap();
        assert_eq!(selected(&f.0).unwrap().path, second.path);
        change(&f.0, "rename", &second.id, Some("Renamed"), None).unwrap();
        assert_eq!(selected(&f.0).unwrap().name, "Renamed");
        assert!(change(&f.0, "create", "", Some("pablo"), None).is_err());
        assert!(change(&f.0, "select", "../escape", None, None).is_err());
    }
    #[test]
    fn upgrade_adopts_custom_original_and_legacy_profiles_in_place() {
        let f = Fixture::new();
        let original = f.original();
        fs::create_dir_all(original.join("savedata")).unwrap();
        fs::write(original.join("savedata/data"), b"synthetic existing save").unwrap();
        let id = saves::create(&f.0, &original, "Legacy copy", Some("original")).unwrap();
        let bid = saves::backup(&f.0, &original, &id).unwrap();
        assert!(list(&f.0, &original).unwrap().has_existing_saves);
        initialize(&f.0, &original, &id, "First owner").unwrap();
        let profiles = list(&f.0, &original).unwrap();
        assert_eq!(profiles.profiles.len(), 2);
        assert_eq!(profiles.active, id);
        assert_eq!(profiles.profiles[0].path, original);
        assert_eq!(profiles.profiles[0].name, "First owner");
        assert_eq!(selected(&f.0).unwrap().backups[0].id, bid);
        assert_eq!(
            fs::read(original.join("savedata/data")).unwrap(),
            b"synthetic existing save"
        );
        change(&f.0, "rename", "original", Some("Renamed owner"), None).unwrap();
        assert_eq!(require(&f.0).unwrap().original_name, "Renamed owner");
        // Config changes after setup cannot silently reassign the first owner's save directory.
        assert_eq!(
            list(&f.0, &f.0.join("other-user")).unwrap().profiles[0].path,
            original
        );
    }
    #[test]
    fn corrupt_registry_never_triggers_fresh_setup_or_fallback() {
        let f = Fixture::new();
        let original = f.original();
        fs::write(f.0.join("profiles.json"), b"not json").unwrap();
        assert!(list(&f.0, &original).is_err());
        assert!(initialize(&f.0, &original, "original", "New").is_err());
        assert_eq!(fs::read(f.0.join("profiles.json")).unwrap(), b"not json");
    }
}
