import { useTranslation } from "react-i18next";
import { ArrowRight, UserRound } from "lucide-react";
import { Button, Card, CardContent } from "@/components/ui";
import { TrophyStatus, TrophySummary } from "@/components/trophies";
import { useProfiles } from "@/hooks";
import { useUiStore } from "@/stores";

/** The identity used by Play, with honest trophy status and links to its management pages. */
export function HomeProfileCard() {
  const { t } = useTranslation();
  const { data } = useProfiles();
  const openProfile = useUiStore((state) => state.openProfile);
  const setTab = useUiStore((state) => state.setTab);
  const profile = data?.profiles.find((entry) => entry.id === data.active);
  if (!profile) return null;

  return (
    <section aria-labelledby="home-profile-title" className="mb-6">
      <Card className="border border-blood-700/40 bg-card/80 shadow-lg shadow-black/20">
        <CardContent className="space-y-3">
          <div className="flex min-w-0 items-start gap-4">
            <div className="flex size-12 shrink-0 items-center justify-center rounded-xl bg-blood-700/20 text-blood-400">
              <UserRound className="size-6" aria-hidden="true" />
            </div>
            <div className="min-w-0">
              <p className="text-xs text-muted-foreground">{t("profiles.active")}</p>
              <div className="mt-1 flex flex-wrap items-center gap-3">
                <h2 id="home-profile-title" className="break-words font-display text-2xl font-semibold text-parchment">{profile.name}</h2>
                <TrophySummary profile={profile} />
              </div>
              <div className="mt-3 flex flex-wrap gap-2">
                <Button variant="secondary" size="sm" onClick={() => setTab("profiles")}>
                  {t("nav.profiles")} <ArrowRight aria-hidden="true" />
                </Button>
                <Button variant="outline" size="sm" onClick={() => openProfile(profile.id)}>{t("nav.saves")}</Button>
              </div>
            </div>
          </div>
          <TrophyStatus profile={profile} />
        </CardContent>
      </Card>
    </section>
  );
}
