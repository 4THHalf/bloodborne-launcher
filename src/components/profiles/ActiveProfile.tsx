import { useTranslation } from "react-i18next";
import { useProfiles } from "@/hooks";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";

export function ActiveProfile() {
  const { t } = useTranslation();
  const { data, change, busy, running } = useProfiles();
  return (
    <div className="mb-3">
      <p className="mb-1 text-xs text-muted-foreground">{t("profiles.active")}</p>
      <Select value={data?.active ?? ""} disabled={busy || running || !data || data.setup_required}
        items={data?.profiles.map((p) => ({ value: p.id, label: p.name })) ?? []}
        onValueChange={(id) => id && change({ action: "select", id }).catch(() => {})}>
        <SelectTrigger className="w-full" aria-label={t("profiles.active")}><SelectValue /></SelectTrigger>
        <SelectContent>{data?.profiles.map((p) => <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>)}</SelectContent>
      </Select>
    </div>
  );
}
