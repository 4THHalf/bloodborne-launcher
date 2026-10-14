import { useTranslation } from "react-i18next";
import { ArrowRight, Trophy } from "lucide-react";
import { Button, Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui";
import { TrophyGroup } from "@/components/trophies/TrophyGroup";
import type { SaveProfile } from "@/lib";

export function TrophySummary({ profile }: { profile: SaveProfile }) {
  const { t } = useTranslation();
  return <div className="space-y-2 text-sm">
    <Dialog>
      <DialogTrigger render={<Button variant="ghost" className="h-auto justify-start gap-3 px-2 py-2" aria-label={t("trophies.view")} />}>
        <Trophy className="size-5 text-parchment/70" aria-hidden="true" />
        <span>{t("profiles.trophies")}</span>
        <span className="rounded-md bg-secondary px-2 py-0.5 tabular-nums" aria-label={t("trophies.recordedCount", { count: profile.trophy_unlocks.length })}>{profile.trophy_unlocks.length}</span>
        <ArrowRight className="size-4 text-muted-foreground" aria-hidden="true" />
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{profile.name} · {t("profiles.trophies")}</DialogTitle>
          <DialogDescription>{t("trophies.local")}</DialogDescription>
        </DialogHeader>
        {!profile.trophies_eligible && <p className="text-sm text-blood-400">{t("trophies.edited")}</p>}
        <div className="max-h-96 space-y-5 overflow-y-auto">
          <TrophyGroup group="base" unlocks={profile.trophy_unlocks.filter((u) => u.group === "base")} />
          <TrophyGroup group="old_hunters" unlocks={profile.trophy_unlocks.filter((u) => u.group === "old_hunters")} />
          {profile.trophy_unlocks.some((u) => !u.group) && <TrophyGroup group="other" unlocks={profile.trophy_unlocks.filter((u) => !u.group)} />}
        </div>
      </DialogContent>
    </Dialog>
    {!profile.trophies_eligible && <p className="text-blood-400">{t("trophies.edited")}</p>}
    {profile.trophy_error && <p role="alert" className="text-destructive">{t("trophies.error")} {profile.trophy_error}</p>}
  </div>;
}
