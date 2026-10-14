import { useTranslation } from "react-i18next";
import { Trophy } from "lucide-react";
import { Button } from "@/components/ui";
import { useUiStore } from "@/stores";
import type { SaveProfile } from "@/lib";

export function TrophySummary({ profile, large = false }: { profile: SaveProfile; large?: boolean }) {
  const { t } = useTranslation();
  const openProfile = useUiStore((state) => state.openProfile);
  return <Button variant="ghost" size="sm" className={large ? "h-auto gap-3 px-2 py-3 text-parchment/80" : "gap-2 text-parchment/70"}
    aria-label={`${t("trophies.view")} · ${t("trophies.recordedCount", { count: profile.trophy_unlocks.length })}`}
    onClick={() => openProfile(profile.id, "trophies")}>
    <Trophy className={large ? "size-9" : "size-4"} aria-hidden="true" />
    <span className={large ? "text-3xl leading-none tabular-nums" : "tabular-nums"}>{profile.trophy_unlocks.length}</span>
  </Button>;
}
