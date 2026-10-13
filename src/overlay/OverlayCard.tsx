import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { TrophyUnlock } from "@/lib";
import { trophyText } from "@/lib/trophy-text";
import { TrophyEmblem } from "@/components/trophy-emblem";

type Notice = Pick<TrophyUnlock, "name" | "description" | "translations" | "grade" | "group"> & { v: number; seq: number; id: number; duration_ms: number; preview: boolean };
export function OverlayCard() {
  const { t, i18n } = useTranslation();
  const [notice, setNotice] = useState<Notice>();
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<Notice>("trophy-notification", ({ payload }) => { if (!disposed) setNotice(payload); })
      .then(async (stop) => { if (disposed) { stop(); return; } unlisten = stop; const language = await invoke<string>("overlay_language"); await i18n.changeLanguage(language); document.documentElement.lang = i18n.resolvedLanguage ?? "en"; await invoke("overlay_ready"); })
      .catch(console.error);
    return () => { disposed = true; unlisten?.(); };
  }, [i18n]);
  if (!notice) return null;
  const text = trophyText(notice, i18n.resolvedLanguage ?? "en");
  return <div key={notice.seq} className={`overlay-card overlay-${notice.grade}`} style={{ animationDuration: `${notice.duration_ms}ms` }} role="status">
    <div className="overlay-emblem"><TrophyEmblem grade={notice.grade} /></div>
    <div><p className="overlay-heading">{notice.preview ? t("trophies.preview") : t("trophies.earned")}</p><p className="overlay-name">{text.name ?? t("trophies.id", { id: notice.id })}</p>{text.description && <p className="overlay-description">{text.description}</p>}<p className="overlay-detail">{t(`trophies.${notice.grade}`)}{notice.group && <> · {t(`trophies.${notice.group}`)}</>}</p></div>
  </div>;
}
