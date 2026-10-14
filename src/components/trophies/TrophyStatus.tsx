import { useTranslation } from "react-i18next";
import type { SaveProfile } from "@/lib";

export function TrophyStatus({ profile }: { profile: SaveProfile }) {
  const { t } = useTranslation();
  return <>
    {!profile.trophies_eligible && <p className="mt-2 text-sm text-blood-400">{t("trophies.edited")}</p>}
    {profile.trophy_error && <p role="alert" className="mt-2 text-sm text-destructive">{t("trophies.error")} {profile.trophy_error}</p>}
  </>;
}
