import { useTranslation } from "react-i18next";
import { FilePenLine, RefreshCw } from "lucide-react";
import { Badge, Button } from "@/components/ui";
import { PageHeader } from "@/components/settings";
import { useSaves } from "@/hooks";
import { SaveProfileCard } from "@/pages/saves/SaveProfileCard";

export function SavesPage() {
  const { t } = useTranslation();
  const { data, error, refetch, isFetching, change, busy, running } = useSaves();
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
      {data?.profiles.map((profile) => (
        <SaveProfileCard key={profile.id} profile={profile} disabled={disabled} onChange={change} />
      ))}
      <div className="flex flex-wrap items-center gap-2 px-1 text-xs text-muted-foreground">
        <FilePenLine className="size-4" aria-hidden="true" />
        <span>{t("saves.editor")}</span>
        <Badge variant="outline">{t("profiles.inDevelopment")}</Badge>
      </div>
    </>
  );
}
