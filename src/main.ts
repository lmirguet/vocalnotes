import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import type { DocumentDispatch } from "./ui/DocumentCommands";
import { mountPreferences } from "./ui/Preferences";
import { loadStartup, renderStartup } from "./ui/Startup";
import { mountDesktopShell } from "./ui/DesktopShell";
import "./style.css";

const root = document.querySelector<HTMLElement>("#app");
if (root) {
  if (isTauri()) {
    try {
      const label = getCurrentWindow().label;
      if (label === "preferences") await mountPreferences(root, invoke);
      else if (label === "main") await mountDesktopShell(root, invoke, {
        listen: callback => listen<DocumentDispatch>("document-command", event => callback(event.payload)),
      });
      else renderStartup(root, { kind: "unavailable" });
    }
    catch { renderStartup(root, { kind: "unavailable" }); }
  } else {
    renderStartup(root, await loadStartup(false, invoke));
  }
}
