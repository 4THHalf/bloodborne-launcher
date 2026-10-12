import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Archive, FolderOpen } from "lucide-react";
import { Badge, Button } from "@/components/ui";
import { SelectRow } from "@/components/settings";
import { api } from "@/lib";
import type { SaveChange, SaveProfile } from "@/lib";

export function SaveProfileCard({ profile, active, disabled, onChange }: {
  profile: SaveProfile; active: boolean; disabled: boolean; onChange: (change: SaveChange) => Promise<void>;
}) {
  const { t, i18n } = useTranslation();
  const [backup, setBackup] = useState("");
  const [confirm, setConfirm] = useState(false);
  const selected = profile.backups.find((b) => b.id === backup)?.id ?? profile.backups[0]?.id ?? "";
  const date = (seconds: number) => new Date(seconds * 1000).toLocaleString(i18n.language);
  const size = (bytes: number) => t("saves.size", { size: (bytes / 1_000_000).toLocaleString(i18n.language, { maximumFractionDigits: 1 }) });
  const run = (change: SaveChange) => onChange(change).then(() => setConfirm(false)).catch(() => {});
  return (
    <div className="py-4">
      <div className="flex flex-wrap items-center gap-3 px-5">
        <Archive className="size-5 text-blood-400" />
        <div className="min-w-0 flex-1">
          <h2 className="font-medium">{profile.name}</h2>
          <p className="mt-1 text-xs text-muted-foreground">
            {profile.bytes ? size(profile.bytes) : t("saves.empty")}
            {profile.modified !== null && <> · {date(profile.modified)}</>}
          </p>
        </div>
        {active && <Badge>{t("saves.active")}</Badge>}
        <Button size="sm" variant="secondary" disabled={disabled} onClick={() => run({ action: "backup", id: profile.id })}>{t("saves.backup")}</Button>
        <Button variant="ghost" size="icon-sm" title={t("common.showInFinder")} onClick={() => api.openPath(profile.path)}><FolderOpen /></Button>
      </div>
      {profile.backups.length > 0 && <>
        <SelectRow title={t("saves.backups")} value={selected} disabled={disabled} onChange={(id) => { setBackup(id); setConfirm(false); }} options={profile.backups.map((b) => ({ value: b.id, label: `${date(b.created)} · ${size(b.bytes)}`, literal: true }))} />
        <div className="flex flex-wrap items-center justify-end gap-3 px-5">
          {confirm && <><p className="flex-1 text-xs text-muted-foreground">{t("saves.restoreHint")}</p><Button variant="ghost" size="sm" onClick={() => setConfirm(false)}>{t("saves.cancel")}</Button></>}
          <Button variant={confirm ? "destructive" : "secondary"} size="sm" disabled={disabled} onClick={() => confirm ? run({ action: "restore", id: profile.id, backup: selected }) : setConfirm(true)}>{t("saves.restore")}</Button>
        </div>
      </>}
    </div>
  );
}
