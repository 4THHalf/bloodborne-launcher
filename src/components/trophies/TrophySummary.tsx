import { useTranslation } from "react-i18next";
import { Trophy } from "lucide-react";
import { Button } from "@/components/ui";
import { useUiStore } from "@/stores";
import type { SaveProfile } from "@/lib";

export function TrophySummary({ profile }: { profile: SaveProfile }) {
  const { t } = useTranslation();
  const openProfile = useUiStore((state) => state.openProfile);
  return <Button variant="ghost" size="sm" className="gap-2 text-parchment/70"
    aria-label={`${t("trophies.view")} · ${t("trophies.recordedCount", { count: profile.trophy_unlocks.length })}`}
    onClick={() => openProfile(profile.id, "trophies")}>
    <Trophy className="size-4" aria-hidden="true" />
    <span className="tabular-nums">{profile.trophy_unlocks.length}</span>
  </Button>;
}
