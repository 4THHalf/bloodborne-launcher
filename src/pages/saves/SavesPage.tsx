import { useTranslation } from "react-i18next";
import { FilePenLine, RefreshCw } from "lucide-react";
import { Badge, Button } from "@/components/ui";
import { FolderRow, PageHeader, SettingsSection } from "@/components/settings";
import { useConfig, useSaves } from "@/hooks";
import { TrophySummary } from "@/components/trophies";
import { SaveProfileCard } from "@/pages/saves/SaveProfileCard";

export function SavesPage() {
  const { t } = useTranslation();
  const { data, error, refetch, isFetching, change, busy, running } = useSaves();
  const { data: config } = useConfig();
  const disabled = running || busy || isFetching;
  const selected = data?.profiles[0];
  return (
    <>
      <PageHeader title={t("saves.title")} subtitle={selected?.name} actions={
        <Button variant="secondary" size="sm" disabled={busy} onClick={() => refetch()}>
          <RefreshCw className={isFetching ? "animate-spin" : undefined} /> {t("common.refresh")}
        </Button>
      } />
      {running && <p className="mb-4 text-sm text-blood-400">{t("saves.stopFirst")}</p>}
      {error && <p role="alert" className="mb-4 text-sm text-destructive">{String(error)}</p>}
      <SettingsSection title={t("saves.title")} description={t("saves.backupHint")}>
        {data?.profiles.map((profile) => (
          <SaveProfileCard key={profile.id} profile={profile} active={data.active === profile.id} disabled={disabled} onChange={change} />
        ))}
      </SettingsSection>

      {selected && <SettingsSection><div className="px-5 py-4"><TrophySummary profile={selected} /></div></SettingsSection>}
      <SettingsSection>
        <div className="flex items-center gap-3 px-5 py-4 text-sm text-muted-foreground">
          <FilePenLine className="size-5" aria-hidden="true" />
          <span>{t("saves.editor")}</span>
          <Badge variant="outline">{t("profiles.inDevelopment")}</Badge>
        </div>
      </SettingsSection>
      <SettingsSection>
        <FolderRow title={t("saves.backups")} path={data?.profiles.some((p) => p.backups.length > 0) ? `${config?.paths.data}/save-backups/${data?.active}` : ""} />
      </SettingsSection>
    </>
  );
}
