import { useTranslation } from "react-i18next";
import { TrophyEmblem } from "@/components/trophy-emblem";
import { trophyText } from "@/lib/trophy-text";
import type { TrophyUnlock } from "@/lib";

export function TrophyGroup({ group, unlocks }: { group: "base" | "old_hunters" | "other"; unlocks: TrophyUnlock[] }) {
  const { t, i18n } = useTranslation();
  return <section className="space-y-2">
    <h3 className="font-medium">{t(`trophies.${group}`)} <span className="text-muted-foreground">({unlocks.length}{group === "base" ? "/34" : group === "old_hunters" ? "/6" : ""})</span></h3>
    {unlocks.length === 0 ? <p className="text-sm text-muted-foreground">{t("trophies.empty")}</p> :
      <ul className="divide-y divide-border">{unlocks.map((unlock) => { const text = trophyText(unlock, i18n.resolvedLanguage ?? "en"); return <li key={`${unlock.title_id}:${unlock.trophy_id}`} className="space-y-1 py-3 text-sm">
        <div className="flex items-start justify-between gap-3"><span className="font-medium">{text.name ?? t("trophies.id", { id: unlock.trophy_id })}</span>
          <span className={`trophy-grade trophy-grade-${unlock.grade} inline-flex items-center gap-1`}><TrophyEmblem grade={unlock.grade} size={14} />{t(`trophies.${unlock.grade}`)}</span></div>
        {text.description && <p className="text-muted-foreground">{text.description}</p>}
        <div className="flex justify-between gap-3 text-xs text-muted-foreground"><span>{unlock.title_id}</span><time dateTime={new Date(unlock.unlocked_at * 1000).toISOString()}>{new Date(unlock.unlocked_at * 1000).toLocaleString(i18n.resolvedLanguage)}</time></div>
      </li>; })}</ul>}
  </section>;
}
