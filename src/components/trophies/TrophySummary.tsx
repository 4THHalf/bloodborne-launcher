import { useTranslation } from "react-i18next";
import { Trophy } from "lucide-react";
import { Button, Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui";
import type { SaveProfile } from "@/lib";

export function TrophySummary({ profile }: { profile: SaveProfile }) {
  const { t, i18n } = useTranslation();
  return <div className="space-y-2 text-sm">
    <div className="flex items-center gap-2"><Trophy className="size-4" aria-hidden="true" />
      <span>{t("trophies.recordedCount", { count: profile.trophy_unlocks.length })}</span>
    </div>
    {!profile.trophies_eligible && <p className="text-blood-400">{t("trophies.edited")}</p>}
    {profile.trophy_error && <p role="alert" className="text-destructive">{t("trophies.error")} {profile.trophy_error}</p>}
    <Dialog>
      <DialogTrigger render={<Button variant="outline" size="sm" />}>{t("trophies.view")}</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{profile.name} · {t("profiles.trophies")}</DialogTitle>
          <DialogDescription>{t("trophies.local")}</DialogDescription>
        </DialogHeader>
        {!profile.trophies_eligible && <p className="text-sm text-blood-400">{t("trophies.edited")}</p>}
        <div className="max-h-80 overflow-y-auto">
          {profile.trophy_unlocks.length === 0 ? <p className="text-sm text-muted-foreground">{t("trophies.empty")}</p> :
            <ul className="divide-y divide-border">{profile.trophy_unlocks.map((unlock) => <li key={`${unlock.title_id}:${unlock.trophy_id}`} className="flex justify-between gap-4 py-3 text-sm">
              <span>{t("trophies.id", { id: unlock.trophy_id })}<span className="block text-xs text-muted-foreground">{unlock.title_id}</span></span>
              <time dateTime={new Date(unlock.unlocked_at * 1000).toISOString()}>{new Date(unlock.unlocked_at * 1000).toLocaleString(i18n.resolvedLanguage)}</time>
            </li>)}</ul>}
        </div>
      </DialogContent>
    </Dialog>
  </div>;
}
