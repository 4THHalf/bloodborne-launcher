import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Trophy } from "lucide-react";

type Notice = { v: number; seq: number; id: number; duration_ms: number };
export function OverlayCard() {
  const { t } = useTranslation();
  const [notice, setNotice] = useState<Notice>();
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<Notice>("prototype-notification", ({ payload }) => { if (!disposed) setNotice(payload); })
      .then(async (stop) => { if (disposed) { stop(); return; } unlisten = stop; await invoke("overlay_ready"); })
      .catch(console.error);
    return () => { disposed = true; unlisten?.(); };
  }, []);
  if (!notice) return null;
  return <div key={notice.seq} className="overlay-card" style={{ animationDuration: `${notice.duration_ms}ms` }} role="status">
    <div className="overlay-emblem"><Trophy size={30} aria-hidden="true" /></div>
    <div><p className="overlay-heading">{t("trophies.preview")}</p><p className="overlay-name">{t("trophies.id", { id: notice.id })}</p></div>
  </div>;
}
