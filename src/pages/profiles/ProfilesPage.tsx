import { useTranslation } from "react-i18next";
import { PageHeader, SettingsSection } from "@/components/settings";
import { useProfiles } from "@/hooks";
import { ProfileCard } from "@/pages/profiles/ProfileCard";
import { CreateProfileDialog } from "@/pages/profiles/CreateProfileDialog";

export function ProfilesPage() {
  const { t } = useTranslation();
  const { data, change, busy, running, error } = useProfiles();
  const disabled = busy || running;
  return (<>
    <PageHeader title={t("profiles.title")} subtitle={t("profiles.subtitle")} />
    {running && <p className="mb-4 text-sm text-blood-400">{t("saves.stopFirst")}</p>}
    {error && <p role="alert">{String(error)}</p>}
    <div className="mb-4">
      <CreateProfileDialog disabled={disabled || !data} busy={busy}
        onCreate={(name) => change({ action: "create", id: "", name })} />
    </div>
    <SettingsSection title={t("profiles.title")}>
      {data?.profiles.map((profile) => <ProfileCard key={profile.id} profile={profile} active={profile.id === data.active} disabled={disabled} onChange={change} />)}
    </SettingsSection>
  </>);
}
