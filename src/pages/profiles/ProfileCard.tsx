import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Trophy } from "lucide-react";
import { Badge, Button, Input, Label } from "@/components/ui";
import { DeleteProfileDialog } from "@/pages/profiles/DeleteProfileDialog";
import type { ProfileChange, SaveProfile } from "@/lib";

export function ProfileCard({ profile, active, disabled, busy, onChange }: {
  profile: SaveProfile; active: boolean; disabled: boolean; busy: boolean; onChange: (change: ProfileChange) => Promise<void>;
}) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(profile.name);
  return (<div className="px-5 py-4">
    <div className="flex items-center gap-3">
      <h2 className="flex-1 font-medium">{profile.name}</h2>
      {active ? <Badge>{t("saves.active")}</Badge> : <Button size="sm" disabled={disabled} onClick={() => onChange({ action: "select", id: profile.id }).catch(() => {})}>{t("profiles.use")}</Button>}
      <Button size="sm" variant="secondary" disabled={disabled} onClick={() => { setName(profile.name); setEditing(true); }}>{t("profiles.rename")}</Button>
      <DeleteProfileDialog profile={profile} active={active} disabled={disabled} busy={busy}
        onDelete={() => onChange({ action: "delete", id: profile.id })} />
    </div>
    <div className="mt-3 flex items-center gap-2 text-sm text-muted-foreground">
      <Trophy className="size-4" aria-hidden="true" />
      <span>{t("profiles.trophies")}</span>
      <Badge variant="outline">{t("profiles.inDevelopment")}</Badge>
    </div>
    {editing && <form className="mt-3 flex items-center gap-3" onSubmit={async (event) => {
      event.preventDefault();
      try { await onChange({ action: "rename", id: profile.id, name: name.trim() }); setEditing(false); } catch { /* Reported by hook. */ }
    }}>
      <Label className="sr-only" htmlFor={`rename-${profile.id}`}>{t("saves.name")}</Label>
      <Input id={`rename-${profile.id}`} maxLength={80} required value={name} disabled={disabled} onChange={(e) => setName(e.target.value)} />
      <Button type="submit" size="sm" disabled={disabled || !name.trim()}>{t("profiles.rename")}</Button>
      <Button type="button" variant="ghost" size="sm" onClick={() => setEditing(false)}>{t("saves.cancel")}</Button>
    </form>}
  </div>);
}
