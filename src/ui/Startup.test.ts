// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { loadStartup, renderStartup } from "./Startup";

describe("socle : état natif sans effet de bord", () => {
  it("n'invoque pas IPC depuis un navigateur", async () => {
    const invoke = vi.fn();
    const state = await loadStartup(false, invoke);
    expect(invoke).not.toHaveBeenCalled();
    expect(state.kind).toBe("browser");
  });

  it("ne demande que l'état du socle, jamais une capture", async () => {
    const invoke = vi.fn().mockResolvedValue({
      stage: "scaffold", capture: "stopped", persistence: "unavailable",
    });
    expect((await loadStartup(true, invoke)).kind).toBe("native");
    expect(invoke.mock.calls).toEqual([["startup_status"]]);
  });

  it("expurge une erreur IPC plutôt que d'afficher le détail reçu", async () => {
    const state = await loadStartup(true, vi.fn().mockRejectedValue(
      new Error("DÉTAIL SYNTHÉTIQUE À NE PAS AFFICHER"),
    ));
    expect(state).toEqual({ kind: "unavailable" });
    const root = document.createElement("main");
    renderStartup(root, state);
    expect(root.textContent).not.toContain("DÉTAIL SYNTHÉTIQUE");
    expect(root.textContent).toContain("indisponible");
  });

  it.each([
    null,
    {},
    { stage: "scaffold", capture: "listening", persistence: "unavailable" },
    { stage: "scaffold", capture: "stopped", persistence: "saved" },
  ])("refuse un état natif incompatible : %j", async (response) => {
    expect(await loadStartup(true, vi.fn().mockResolvedValue(response)))
      .toEqual({ kind: "unavailable" });
  });

  it("annonce les limites web sans prétendre une recette Tauri", async () => {
    const root = document.createElement("main");
    renderStartup(root, await loadStartup(false, vi.fn()));
    expect(root.querySelector("[role=status]")?.textContent).toContain("Aperçu web");
    expect(root.textContent).toContain("Aucune capture");
    expect(root.textContent).toContain("non validé");
    expect(root.querySelector("textarea, [contenteditable=true]")).toBeNull();
    expect(root.querySelector("button")).toBeNull();
  });

  it("n'annonce aucun stockage ou service comme fonctionnel", () => {
    const root = document.createElement("main");
    renderStartup(root, { kind: "native" });
    expect(root.textContent).toContain("prototype non réalisé");
    expect(root.textContent).toContain("Sauvegarde et services non disponibles");
    expect(root.querySelector("iframe, img, a, audio")).toBeNull();
  });
});
