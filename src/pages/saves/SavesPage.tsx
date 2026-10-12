import { useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw } from "lucide-react";
import { Button, Input, Label } from "@/components/ui";
import { FolderRow, PageHeader, SelectRow, SettingsSection } from "@/components/settings";
import { useConfig, useSaves, useSetting } from "@/hooks";
import { SaveProfileCard } from "@/pages/saves/SaveProfileCard";

export function SavesPage() {
  const { t } = useTranslation();
  const { data, error, refetch, isFetching, change, busy, running } = useSaves();
  const { data: config } = useConfig();
  const setSetting = useSetting();
  const [name, setName] = useState("");
  const [source, setSource] = useState("fresh");
  const disabled = running || busy || isFetching;
  const original = data?.profiles.find((p) => p.id === "original");
  return (
    <>
      <PageHeader title={t("saves.title")} subtitle={t("saves.subtitle")} actions={
        <Button variant="secondary" size="sm" disabled={busy} onClick={() => refetch()}>
          <RefreshCw className={isFetching ? "animate-spin" : undefined} /> {t("common.refresh")}
        </Button>
      } />
      {running && <p className="mb-4 text-sm text-blood-400">{t("saves.stopFirst")}</p>}
      {error && <p role="alert" className="mb-4 text-sm text-destructive">{String(error)}</p>}
      <SettingsSection title={t("saves.profiles")} description={t("saves.backupHint")}>
        {data?.profiles.map((profile) => (
          <SaveProfileCard key={profile.id} profile={profile} active={data.active === profile.id} disabled={disabled} onChange={change} />
        ))}
      </SettingsSection>
      <SettingsSection title={t("saves.newProfile")}>
        <form onSubmit={async (event) => {
          event.preventDefault();
          if (disabled || !data) return;
          try {
            await change({ action: source === "fresh" ? "create" : "duplicate", id: source, name: name.trim() });
            setName("");
          } catch { /* The hook reports the backend error. */ }
        }}>
          <div className="flex items-center gap-4 px-5 py-4">
            <Label htmlFor="save-profile-name" className="shrink-0">{t("saves.name")}</Label>
            <Input id="save-profile-name" maxLength={80} required value={name} disabled={disabled || !data} onChange={(e) => setName(e.target.value)} />
            <Button type="submit" disabled={disabled || !data || !name.trim()}>{t("saves.create")}</Button>
          </div>
          <SelectRow title={t("saves.startFrom")} value={source} disabled={disabled || !data} onChange={setSource} options={[
            { value: "fresh", label: "saves.fresh" },
            ...(data?.profiles ?? []).map((p) => ({ value: p.id, label: p.id === "original" ? t("saves.original") : p.name, literal: true as const })),
          ]} />
        </form>
      </SettingsSection>
      <SettingsSection>
        <FolderRow title={t("saves.backups")} path={data?.profiles.some((p) => p.backups.length > 0) ? `${config?.paths.data}/save-backups` : ""} />
        <FolderRow title={t("saves.originalFolder")} path={original?.path ?? ""} custom={!!config?.settings.user_dir}
          onChoose={disabled ? undefined : (path) => setSetting("user_dir", path)}
          onReset={disabled ? undefined : () => setSetting("user_dir", "")} />
      </SettingsSection>
    </>
  );
}
