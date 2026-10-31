export type StartupState =
  | { kind: "browser" }
  | { kind: "native" }
  | { kind: "unavailable" };

type InvokeStatus = (command: string) => Promise<unknown>;

export async function loadStartup(
  native: boolean,
  invoke: InvokeStatus,
): Promise<StartupState> {
  if (!native) return { kind: "browser" };
  try {
    const response = await invoke("startup_status");
    if (
      typeof response === "object" && response !== null &&
      "stage" in response && response.stage === "scaffold" &&
      "capture" in response && response.capture === "stopped" &&
      "persistence" in response && response.persistence === "unavailable"
    ) {
      return { kind: "native" };
    }
  } catch {
    // Neither exception details nor native payloads enter the DOM or logs.
  }
  return { kind: "unavailable" };
}

export function renderStartup(root: HTMLElement, state: StartupState): void {
  const heading = document.createElement("h1");
  heading.textContent = "Bloc-notes vocal";
  const status = document.createElement("p");
  status.setAttribute("role", "status");
  status.textContent = {
    browser: "Aperçu web du socle — Tauri non validé sur ce navigateur.",
    native: "Socle natif — prototype non réalisé.",
    unavailable: "Cœur natif indisponible — aucune opération effectuée.",
  }[state.kind];
  const scope = document.createElement("p");
  scope.textContent =
    "Aucune capture ni requête externe. Sauvegarde et services non disponibles.";
  const gate = document.createElement("p");
  gate.textContent =
    "T005 en validation. T006 et le jalon GNOME/Wayland + Windows 11 restent à réaliser.";
  root.replaceChildren(heading, status, scope, gate);
}
