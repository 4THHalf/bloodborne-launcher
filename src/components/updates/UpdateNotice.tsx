import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink, X } from "lucide-react";
import { Alert, AlertDescription, AlertTitle, Button } from "@/components/ui";
import { useOpenRelease, useRelease } from "@/hooks";

export function UpdateNotice() {
  const { t } = useTranslation();
  const { data: release } = useRelease();
  const open = useOpenRelease();
  const [dismissed, setDismissed] = useState<string | null>(null);
  if (!release?.newer || dismissed === release.tag) return null;
  return (
    <Alert className="mb-5 border-primary/30">
      <AlertTitle>{t("updates.available", { version: release.version })}</AlertTitle>
      <AlertDescription className="flex flex-wrap items-center justify-between gap-3">
        <span>{t("updates.downloadHint")}</span>
        <div className="flex items-center gap-2">
          <Button size="sm" disabled={open.isPending} onClick={() => open.mutate(release.tag)}>
            <ExternalLink />{t("updates.viewRelease")}
          </Button>
          <Button size="icon-sm" variant="ghost" aria-label={t("updates.dismiss")} onClick={() => setDismissed(release.tag)}><X /></Button>
        </div>
      </AlertDescription>
    </Alert>
  );
}
