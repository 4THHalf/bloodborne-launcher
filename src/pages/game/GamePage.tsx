import { useTranslation } from "react-i18next";
import { FolderRow, PageHeader, SelectRow, SettingsSection, SwitchRow } from "@/components/settings";
import { useConfig, useIni, useSetting, useSettingsData } from "@/hooks";
import { GAME_LANGUAGES } from "@/lib/data";

export function GamePage() {
  const { t } = useTranslation();
  const { settings, ini } = useSettingsData();
  const { data: config } = useConfig();
  const setIni = useIni();
  const setSetting = useSetting();
  const flag = (key: string, fallback: boolean) => (key in ini ? ini[key] === "1" : fallback);
  return (
    <>
      <PageHeader title={t("game.title")} subtitle={t("game.subtitle")} />
      <SettingsSection title={t("game.language")}>
        <SelectRow
          title={t("game.systemLanguage")}
          description={t("game.systemLanguageHint")}
          value={settings.language}
          options={GAME_LANGUAGES}
          onChange={(v) => setSetting("language", v)}
        />
      </SettingsSection>
      <SettingsSection title={t("game.extras")} description={t("common.restartNote")}>
        <SwitchRow title={t("game.skipIntro")} checked={flag("skip_intro", false)} onChange={(v) => setIni("skip_intro", v ? "1" : "0")} />
      </SettingsSection>
      <SettingsSection title={t("game.saves")}>
        <FolderRow title={t("game.savesFolder")} path={config?.paths.saves ?? ""} />
      </SettingsSection>
    </>
  );
}
