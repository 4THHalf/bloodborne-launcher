import { useTranslation } from "react-i18next";
import { ArrowRight, UserRound } from "lucide-react";
import { Button, Card, CardContent } from "@/components/ui";
import { TrophySummary } from "@/components/trophies";
import { useProfiles } from "@/hooks";
import { useUiStore } from "@/stores";

/** The identity used by Play, with honest trophy status and links to its management pages. */
export function HomeProfileCard() {
  const { t } = useTranslation();
  const { data } = useProfiles();
  const setTab = useUiStore((state) => state.setTab);
  const profile = data?.profiles.find((entry) => entry.id === data.active);
  if (!profile) return null;

  return (
    <section aria-labelledby="home-profile-title" className="mb-6">
      <Card className="border border-blood-700/40 bg-card/80 shadow-lg shadow-black/20">
        <CardContent className="grid gap-5 md:grid-cols-[minmax(0,1fr)_auto]">
          <div className="flex min-w-0 items-start gap-4">
            <div className="flex size-12 shrink-0 items-center justify-center rounded-xl bg-blood-700/20 text-blood-400">
              <UserRound className="size-6" aria-hidden="true" />
            </div>
            <div className="min-w-0">
              <p className="text-xs text-muted-foreground">{t("profiles.active")}</p>
              <h2 id="home-profile-title" className="mt-1 break-words font-display text-2xl font-semibold text-parchment">{profile.name}</h2>
              <div className="mt-3 flex flex-wrap gap-2">
                <Button variant="secondary" size="sm" onClick={() => setTab("profiles")}>
                  {t("nav.profiles")} <ArrowRight aria-hidden="true" />
                </Button>
                <Button variant="outline" size="sm" onClick={() => setTab("saves")}>{t("nav.saves")}</Button>
              </div>
            </div>
          </div>
          <div className="flex items-center gap-4 border-t border-border pt-4 md:border-l md:border-t-0 md:pl-6 md:pt-0">
            <TrophySummary profile={profile} />
          </div>
        </CardContent>
      </Card>
    </section>
  );
}
