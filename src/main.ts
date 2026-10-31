import { invoke, isTauri } from "@tauri-apps/api/core";
import { loadStartup, renderStartup } from "./ui/Startup";
import { mountDesktopShell } from "./ui/DesktopShell";
import "./style.css";

const root = document.querySelector<HTMLElement>("#app");
if (root) {
  if (isTauri()) {
    try { await mountDesktopShell(root, invoke); }
    catch { renderStartup(root, { kind: "unavailable" }); }
  } else {
    renderStartup(root, await loadStartup(false, invoke));
  }
}
