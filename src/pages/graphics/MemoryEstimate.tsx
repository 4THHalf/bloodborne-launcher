import { useTranslation } from "react-i18next";
import { SettingRow } from "@/components/settings";
import { useConfig, useSettingsData } from "@/hooks";
import { MEMORY_WARNING_SHARE, estimateMemoryMb } from "@/lib/data";
import { cn } from "@/lib";

/** The memory the game will use at its next start with these settings, next to the computer's. */
export function MemoryEstimate() {
  const { t } = useTranslation();
  const { ini } = useSettingsData();
  const { data: config } = useConfig();
  const used = estimateMemoryMb(ini);
  const total = config?.memory_mb ?? 0;
  const high = total > 0 && used > total * MEMORY_WARNING_SHARE;
  const gb = (mb: number) => (mb / 1024).toFixed(1);
  return (
    <SettingRow
      title={t("graphics.memory")}
      description={high ? t("graphics.memoryHigh") : t("graphics.memoryHint")}
    >
      <span className={cn("text-sm tabular-nums", high ? "text-amber-400" : "text-muted-foreground")}>
        {total > 0 ? t("graphics.memoryOf", { used: gb(used), total: gb(total) }) : t("graphics.memoryGb", { used: gb(used) })}
      </span>
    </SettingRow>
  );
}
