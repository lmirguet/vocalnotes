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

  describe("montage depuis le label natif réel", () => {
    it.each(["main", "preferences", "unknown"])("ignore la query string pour %s", async label => {
      vi.resetModules();
      window.history.replaceState({}, "", "?window=preferences&label=main");
      const root = document.createElement("main"); root.id = "app"; document.body.replaceChildren(root);
      const invoke = vi.fn(), main = vi.fn(), preferences = vi.fn();
      vi.doMock("@tauri-apps/api/core", () => ({isTauri:() => true,invoke}));
      vi.doMock("@tauri-apps/api/window", () => ({getCurrentWindow:() => ({label})}));
      vi.doMock("./DesktopShell", () => ({mountDesktopShell:main}));
      vi.doMock("./Preferences", () => ({mountPreferences:preferences}));
      try {
        await import("../main");
        expect(main).toHaveBeenCalledTimes(label === "main" ? 1 : 0);
        expect(preferences).toHaveBeenCalledTimes(label === "preferences" ? 1 : 0);
        expect(invoke).not.toHaveBeenCalled();
        if (label === "unknown") expect(root.textContent).toContain("indisponible");
      } finally {
        for (const module of ["@tauri-apps/api/core","@tauri-apps/api/window","./DesktopShell","./Preferences"]) vi.doUnmock(module);
        window.history.replaceState({}, "", "/");
        vi.resetModules();
      }
    });
    it("affiche un échec de création sans texte sensible ni shell de note de secours", async () => {
      vi.resetModules();
      const root = document.createElement("main"); root.id = "app"; document.body.replaceChildren(root);
      const main = vi.fn(), preferences = vi.fn().mockRejectedValue(new Error("DÉTAIL À MASQUER"));
      vi.doMock("@tauri-apps/api/core", () => ({isTauri:() => true,invoke:vi.fn()}));
      vi.doMock("@tauri-apps/api/window", () => ({getCurrentWindow:() => ({label:"preferences"})}));
      vi.doMock("./DesktopShell", () => ({mountDesktopShell:main}));
      vi.doMock("./Preferences", () => ({mountPreferences:preferences}));
      try {
        await import("../main");
        expect(main).not.toHaveBeenCalled();
        expect(root.textContent).toContain("indisponible");
        expect(root.textContent).not.toContain("DÉTAIL À MASQUER");
      } finally {
        for (const module of ["@tauri-apps/api/core","@tauri-apps/api/window","./DesktopShell","./Preferences"]) vi.doUnmock(module);
        vi.resetModules();
      }
    });
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
