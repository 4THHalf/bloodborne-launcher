import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, Label } from "@/components/ui";
import { TitleBar } from "@/components/window/TitleBar";
import { useProfiles } from "@/hooks";

/** Mandatory on first install and upgrades without profile ownership. No launcher is behind it. */
export function ProfileSetup() {
  const { t } = useTranslation();
  const { data, error, refetch, change, busy, running } = useProfiles();
  const [name, setName] = useState("");
  const nameInput = useRef<HTMLInputElement>(null);
  useEffect(() => { nameInput.current?.focus(); }, []);
  return (
    <div className="flex h-full flex-col"><TitleBar />
      <section role="dialog" aria-modal="true" aria-labelledby="profile-setup-title" aria-describedby="profile-setup-description"
        className="flex flex-1 items-center justify-center px-8 pb-9">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle id="profile-setup-title" className="font-display text-2xl">{t("profiles.setupTitle")}</CardTitle>
            <CardDescription id="profile-setup-description">{t(data?.has_existing_saves ? "profiles.setupExisting" : "profiles.setupFresh")}</CardDescription>
          </CardHeader>
          <CardContent>
            {error ? <><p role="alert" className="mb-4 text-sm text-destructive">{String(error)}</p><Button onClick={() => refetch()}>{t("common.refresh")}</Button></> :
              <form onSubmit={async (event) => {
                event.preventDefault();
                if (busy || running || !data) return;
                try { await change({ action: "initialize", id: "original", name: name.trim() }); } catch { /* Reported by hook. */ }
              }}>
                <Label htmlFor="first-profile-name">{t("saves.name")}</Label>
                <Input ref={nameInput} className="mb-4 mt-2" id="first-profile-name" maxLength={80} required value={name} disabled={busy || running} onChange={(e) => setName(e.target.value)} />
                {running && <p className="mb-4 text-sm">{t("saves.stopFirst")}</p>}
                <Button type="submit" className="w-full" disabled={busy || running || !data || !name.trim()}>{t("profiles.setupCreate")}</Button>
              </form>}
          </CardContent>
        </Card>
      </section>
    </div>
  );
}
