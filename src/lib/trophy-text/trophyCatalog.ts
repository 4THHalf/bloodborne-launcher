import catalog from "@/lib/trophy-text/trophy_catalog.json";
import type { TrophyUnlock } from "@/lib/types";

// Shared public catalog, also validated by the native catalog tests. Earlier save progress
// is not inferred: an entry is unlocked only when this profile has a matching local event.
export const trophyCatalog: Omit<TrophyUnlock, "unlocked_at">[] = catalog.trophies.map((entry) => ({
  title_id: "CUSA03173",
  trophy_id: entry.id,
  grade: entry.grade as TrophyUnlock["grade"],
  group: entry.group as TrophyUnlock["group"],
  name: entry.translations.en.name,
  description: entry.translations.en.description,
  translations: entry.translations,
}));
