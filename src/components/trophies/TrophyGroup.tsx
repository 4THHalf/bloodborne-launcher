import { useTranslation } from "react-i18next";
import { Check, LockKeyhole } from "lucide-react";
import { TrophyEmblem } from "@/components/trophy-emblem";
import { trophyText, trophyCatalog } from "@/lib/trophy-text";
import type { TrophyUnlock } from "@/lib";

export function TrophyGroup({ group, unlocks }: { group: "base" | "old_hunters" | "other"; unlocks: TrophyUnlock[] }) {
  const { t, i18n } = useTranslation();
  const entries = group === "other" ? unlocks.map((trophy) => ({ trophy, unlocked: trophy })) :
    trophyCatalog.filter((trophy) => trophy.group === group).map((trophy) => ({ trophy,
      unlocked: unlocks.find((unlock) => unlock.title_id === trophy.title_id && unlock.trophy_id === trophy.trophy_id) }));
  const earned = entries.filter((entry) => entry.unlocked).length;
  return <section className="space-y-3">
    <h2 className="font-display text-lg">{t(`trophies.${group}`)} <span className="font-sans text-sm text-muted-foreground">{earned}/{entries.length}</span></h2>
    <ul className="divide-y divide-border">{entries.map(({ trophy, unlocked }) => {
      const text = trophyText(trophy, i18n.resolvedLanguage ?? "en");
      return <li key={`${trophy.title_id}:${trophy.trophy_id}`} className="flex items-start gap-4 py-4 text-sm">
        <div className={`mt-1 shrink-0 ${unlocked ? "" : "opacity-40"}`}><TrophyEmblem grade={trophy.grade} size={28} /></div>
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="font-medium">{text.name ?? t("trophies.id", { id: trophy.trophy_id })}</span>
            <span className={`trophy-grade trophy-grade-${trophy.grade} text-xs`}>{t(`trophies.${trophy.grade}`)}</span>
          </div>
          {text.description && <p className="text-muted-foreground">{text.description}</p>}
          <div className={`flex flex-wrap items-center gap-2 text-xs ${unlocked ? "text-parchment/80" : "text-muted-foreground"}`}>
            {unlocked ? <Check className="size-3" aria-hidden="true" /> : <LockKeyhole className="size-3" aria-hidden="true" />}
            <span>{t(unlocked ? "trophies.unlocked" : "trophies.locked")}</span>
            {unlocked && <time dateTime={new Date(unlocked.unlocked_at * 1000).toISOString()}>{new Date(unlocked.unlocked_at * 1000).toLocaleString(i18n.resolvedLanguage)}</time>}
          </div>
        </div>
      </li>;
    })}</ul>
  </section>;
}
