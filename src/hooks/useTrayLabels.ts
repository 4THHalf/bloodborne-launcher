import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib";

/** Keep native menu-bar actions in the selected launcher language. */
export function useTrayLabels() {
  const { t } = useTranslation();
  useEffect(() => {
    api.setTrayLabels({ show: t("tray.show"), stop: t("tray.stop") })
      .catch((error: unknown) => console.error("Tray menu labels:", error));
  }, [t]);
}
