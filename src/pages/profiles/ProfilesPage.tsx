import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button, Input, Label } from "@/components/ui";
import { PageHeader, SelectRow, SettingsSection } from "@/components/settings";
import { useProfiles } from "@/hooks";
import { ProfileCard } from "@/pages/profiles/ProfileCard";

export function ProfilesPage() {
  const { t } = useTranslation();
  const { data, change, busy, running, error } = useProfiles();
  const [name, setName] = useState("");
  const [source, setSource] = useState("fresh");
  const disabled = busy || running;
  return (<>
    <PageHeader title={t("profiles.title")} subtitle={t("profiles.subtitle")} />
    {running && <p className="mb-4 text-sm text-blood-400">{t("saves.stopFirst")}</p>}
    {error && <p role="alert">{String(error)}</p>}
    <SettingsSection title={t("profiles.title")}>
      {data?.profiles.map((profile) => <ProfileCard key={profile.id} profile={profile} active={profile.id === data.active} disabled={disabled} onChange={change} />)}
    </SettingsSection>
    <SettingsSection title={t("saves.newProfile")}>
      <form onSubmit={async (event) => {
        event.preventDefault();
        if (disabled || !data) return;
        try { await change({ action: "create", id: "", name: name.trim(), source: source === "fresh" ? undefined : source }); setName(""); } catch { /* Reported by hook. */ }
      }}>
        <div className="flex items-center gap-4 px-5 py-4">
          <Label htmlFor="profile-name">{t("saves.name")}</Label>
          <Input id="profile-name" maxLength={80} required value={name} disabled={disabled} onChange={(e) => setName(e.target.value)} />
          <Button type="submit" disabled={disabled || !data || !name.trim()}>{t("saves.create")}</Button>
        </div>
        <SelectRow title={t("saves.startFrom")} value={source} disabled={disabled} onChange={setSource}
          options={[{ value: "fresh", label: "saves.fresh" }, ...(data?.profiles.map((p) => ({ value: p.id, label: p.name, literal: true as const })) ?? [])]} />
      </form>
    </SettingsSection>
  </>);
}
