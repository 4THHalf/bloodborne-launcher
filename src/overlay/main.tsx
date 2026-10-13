import { createRoot } from "react-dom/client";
import "@/index.css";
import { i18n } from "@/i18n";
import "@/overlay/overlay.css";
import { OverlayCard } from "@/overlay/OverlayCard";
// Synthetic acceptance tests use English; live localization is a later protocol feature.
void i18n.changeLanguage("en");
document.documentElement.lang = "en";
document.documentElement.classList.add("dark");
createRoot(document.getElementById("root")!).render(<OverlayCard />);
