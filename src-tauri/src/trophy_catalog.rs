// SPDX-License-Identifier: GPL-2.0-or-later
//! Public trophy names/grades: https://psn100.net/game/3609-bloodborne.
//! IDs cross-checked against Supermedo/bloodborne_pc 79d44f0 (Ambrose).
//! Requirement summaries independently worded using Gamepressure’s public trophy guide.
//! No game artwork, extracted metadata or keys are included.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Platinum,
    Gold,
    Silver,
    Bronze,
    #[default]
    Unknown,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Text {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}
#[derive(Deserialize)]
pub struct Trophy {
    pub id: u8,
    pub grade: Grade,
    pub group: String,
    pub translations: BTreeMap<String, Text>,
}
impl Trophy {
    pub fn text(&self, locale: &str) -> &Text {
        let normalized = locale.replace('_', "-");
        self.translations
            .get(&normalized)
            .or_else(|| {
                self.translations
                    .get(normalized.split('-').next().unwrap_or("en"))
            })
            .unwrap_or_else(|| {
                self.translations
                    .get("en")
                    .expect("checked English fallback")
            })
    }
}
#[derive(Deserialize)]
struct Catalog {
    schema_version: u8,
    default_locale: String,
    trophies: Vec<Trophy>,
    translation_sources: BTreeMap<String, String>,
}
fn catalog() -> &'static Catalog {
    static CATALOG: std::sync::OnceLock<Catalog> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| {
        let catalog: Catalog = serde_json::from_str(include_str!("../../src/lib/trophy-text/trophy_catalog.json"))
            .expect("checked public catalog");
        assert_eq!(catalog.schema_version, 1);
        assert!(catalog.trophies.iter().all(|t| t
            .translations
            .keys()
            .all(|locale| catalog.translation_sources.contains_key(locale))));
        assert_eq!(catalog.default_locale, "en");
        catalog
    })
}
pub fn find(title: &str, id: u8) -> Option<&'static Trophy> {
    // Only the supported release's public ID mapping is verified.
    if title != "CUSA03173" {
        return None;
    }
    catalog().trophies.iter().find(|t| t.id == id)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_catalog_identity_and_grades() {
        assert!(find("SYNTH0001", 0).is_none());
        assert_eq!(catalog().trophies.len(), 40);
        let mut counts = [0; 4];
        for id in 0..40 {
            let t = find("CUSA03173", id).unwrap();
            assert_eq!(t.id, id);
            assert!(!t.text("en").name.is_empty());
            assert!(t
                .text("en")
                .description
                .as_ref()
                .is_some_and(|d| !d.is_empty()));
            assert_eq!(t.text("unverified").name, t.text("en").name);
            assert_eq!(t.group, if id < 34 { "base" } else { "old_hunters" });
            counts[match t.grade {
                Grade::Platinum => 0,
                Grade::Gold => 1,
                Grade::Silver => 2,
                Grade::Bronze => 3,
                Grade::Unknown => panic!("missing grade"),
            }] += 1;
        }
        assert_eq!(counts, [1, 7, 8, 24]);
        assert!(find("CUSA03173", 40).is_none());
        assert_ne!(
            find("CUSA03173", 1).unwrap().text("pt-BR").name,
            find("CUSA03173", 1).unwrap().text("en").name
        );
        assert_eq!(
            find("CUSA03173", 35).unwrap().text("fr").name,
            "Orphelin de Kos"
        );
        // Stable anchors from the public German list catch a shifted import row.
        for (id, name) in [
            (1, "Yharnam-Sonnenaufgang"),
            (14, "Pater Gascoigne"),
            (21, "Kleriker-Bestie"),
            (35, "Waisenknabe von Kos"),
        ] {
            assert_eq!(find("CUSA03173", id).unwrap().text("de").name, name);
        }
        assert_eq!(
            find("CUSA03173", 35).unwrap().text("pt-PT").name,
            find("CUSA03173", 35).unwrap().text("en").name
        );
        for trophy in &catalog().trophies {
            assert_eq!(trophy.translations.len(), catalog().translation_sources.len());
            for text in trophy.translations.values() {
                assert!(!text.name.is_empty());
                assert!(!matches!(
                    text.name.as_str(),
                    "Sony Computer Entertainment America"
                        | "Role-Playing Games (RPG)"
                        | "Japanese"
                        | "Spanish"
                        | "Italian"
                        | "Portuguese"
                        | "Norway"
                        | "Tweet"
                        | "Mike Bendel"
                        | "Valve"
                        | "Mobile App"
                        | "About"
                        | "Legal"
                        | "Terms"
                        | "Threads"
                        | "Rules"
                        | "Contact"
                        | "Privacy Policy"
                        | "Join Premium"
                        | "Twitter"
                ));
            }
        }
    }
}
