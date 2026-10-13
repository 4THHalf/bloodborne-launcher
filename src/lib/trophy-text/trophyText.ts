import type { TrophyUnlock } from "@/lib";

export function trophyText(trophy: Pick<TrophyUnlock, "name" | "description" | "translations">, locale: string) {
  const normalized = locale.replaceAll("_", "-");
  const text = trophy.translations[normalized] ?? trophy.translations[normalized.split("-")[0]];
  return { name: text?.name ?? trophy.name, description: text?.description ?? trophy.description };
}
