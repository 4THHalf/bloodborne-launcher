import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui";
import { SettingRow, SettingsSection, SwitchRow } from "@/components/settings";
import { useConfig, useOpenRelease, useRelease, useSetting } from "@/hooks";

export function UpdateSettings() {
  const { t } = useTranslation();
  const { data: config } = useConfig();
  const release = useRelease();
  const open = useOpenRelease();
  const latest = release.data;
  const setSetting = useSetting();
  const status = release.isFetching ? t("updates.checking")
    : release.isError ? t("updates.unavailable")
    : !release.isFetched ? t("updates.notChecked")
    : release.data === null ? t("updates.noRelease")
    : release.data?.newer ? t("updates.available", { version: release.data.version })
    : t("updates.current");
  return (
    <SettingsSection title={t("updates.title")}>
      <SwitchRow title={t("updates.automatic")} description={t("updates.automaticHint")}
        checked={config?.settings.check_updates ?? true} onChange={(value) => setSetting("check_updates", value)} />
      <SettingRow title={t("updates.installed", { version: config?.version })} description={<span role="status">{status}</span>}>
        {latest?.newer && <Button size="sm" disabled={open.isPending} onClick={() => open.mutate(latest.tag)}>{t("updates.viewRelease")}</Button>}
        <Button variant="outline" size="sm" disabled={release.isFetching} onClick={() => void release.refetch()}>{t("updates.checkNow")}</Button>
      </SettingRow>
    </SettingsSection>
  );
}
