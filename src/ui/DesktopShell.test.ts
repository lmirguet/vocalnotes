// @vitest-environment jsdom
import { describe, it, expect, vi } from "vitest";
import { mountDesktopShell, type Snapshot } from "./DesktopShell";

const initial: Snapshot = { noteId:1, manualPeriod:1, revision:0, generation:1, mode:"manual",
  source:"Note", dirty:false, message:"Prêt", blocks:[{from:0,to:4,raw:"Note",kind:"paragraph"}] };

describe("fenêtre de note", () => {
  it("réunit menus, bandeau et une note sans cible de traitement inventée", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const invoke = vi.fn(async () => initial);
    const app = await mountDesktopShell(root, invoke);
    try {
      expect([...root.querySelectorAll(".menu-trigger")].map(e => e.textContent)).toEqual(["Fichier","Édition","Aide"]);
      expect(root.querySelector('[role="toolbar"]')).not.toBeNull();
      for (const action of ["Gras","Italique","Titre","start","stop"]) expect(root.querySelector(`[data-action="${action}"]`)).not.toBeNull();
      expect(root.querySelectorAll(".ProseMirror")).toHaveLength(1);
      expect(root.querySelector('[data-target], [data-processing-zone]')).toBeNull();
      expect(root.textContent).not.toContain("T006 non recetté");
      expect(root.querySelector('[role="status"]')?.getAttribute("aria-live")).toBe("polite");
      expect(invoke).toHaveBeenCalledExactlyOnceWith("prototype_snapshot", undefined);
    } finally { app.destroy(); }
  });
  it("place le conflit avant les compteurs après un enregistrement refusé", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const conflict = "Conflit disque : versions préservées.\nVersion locale : /tmp/local.md\nVersion disque : /tmp/externe.md";
    let snapshot = initial;
    const app = await mountDesktopShell(root, async command => {
      if (command === "prototype_action") {
        snapshot = {...initial, dirty:true, storageMessage:conflict};
        throw "Conflit disque : enregistrer une nouvelle copie avant de continuer la sauvegarde.";
      }
      return snapshot;
    });
    try {
      root.querySelector<HTMLButtonElement>('[data-action="save-current"]')!.click();
      await vi.waitFor(() => {
        const status = root.querySelector('[role="status"]')!;
        expect(status.firstElementChild?.getAttribute("data-storage-warning")).toBe("true");
        expect(status.firstElementChild?.textContent).toBe(conflict);
        expect(status.textContent).not.toContain("Échec de l'action");
      });
    } finally { app.destroy(); }
  });
  it("efface une clé non envoyée quand les préférences sont refermées", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const invoke = vi.fn(async (command: string) => command === "preferences_get" ? {
      stt:{provider:"mistral",model:"voxtral-mini-transcribe-realtime-2602"},
      llm:{provider:"mistral",model:"mistral-large-latest"}, lastNote:null,
    } : initial);
    const app = await mountDesktopShell(root, invoke);
    try {
      const open = root.querySelector<HTMLButtonElement>('[data-action="preferences"]')!;
      open.click();
      await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("preferences_get"));
      const secret = root.querySelector<HTMLInputElement>('input[type="password"]')!;
      secret.value = "synthetic-unsent-key";
      open.click();
      expect(secret.value).toBe("");
      expect(invoke.mock.calls.every(call => call[0] !== "prototype_set_key")).toBe(true);
      expect(root.textContent).not.toContain("synthetic-unsent-key");
    } finally { app.destroy(); }
  });
  it("interdit la saisie et les commandes de mise en forme pendant le vocal", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const invoke = vi.fn(async () => ({...initial,mode:"voice"}));
    const app = await mountDesktopShell(root, invoke);
    try {
      expect(root.querySelector(".ProseMirror")?.getAttribute("contenteditable")).toBe("false");
      for (const action of ["Gras","Italique","Titre","Paragraphe","list","undo","redo"])
        expect(root.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)?.disabled).toBe(true);
      root.querySelector<HTMLButtonElement>('[data-action="Gras"]')!.click();
      expect(invoke).toHaveBeenCalledTimes(1);
    } finally { app.destroy(); }
  });
  it("met à jour l’état sans déplacer le focus de la destination", async () => {
    vi.useFakeTimers();
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let calls = 0;
    const app = await mountDesktopShell(root, async () => ++calls === 1 ? initial : {...initial,message:"Passage en cours"});
    try {
      root.querySelector<HTMLButtonElement>(".menu-trigger")!.click();
      const path = root.querySelector<HTMLInputElement>('[aria-label="Destination Markdown locale"]')!;
      path.focus();
      await vi.advanceTimersByTimeAsync(1000);
      expect(root.querySelector('[role="status"]')?.textContent).toContain("Passage en cours");
      expect(document.activeElement).toBe(path);
      expect(root.querySelector('[aria-expanded="true"]')).not.toBeNull();
    } finally { app.destroy(); vi.useRealTimers(); }
  });
  it("place l’intention au-dessus de la note et la synchronise sans édition ni focus imposé", async () => {
    vi.useFakeTimers();
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let snapshot: Snapshot = { ...initial, mode: "voice", draftingIntention: null };
    const app = await mountDesktopShell(root, async () => snapshot);
    try {
      const field = root.querySelector<HTMLTextAreaElement>('[aria-label="Intention de rédaction courante"]')!;
      const editor = root.querySelector('[aria-label="Note"]')!;
      expect(field.compareDocumentPosition(editor) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      const focused = root.querySelector<HTMLButtonElement>(".menu-trigger")!; focused.focus();
      snapshot = { ...snapshot, draftingIntention: { purpose: "Présenter un devis", form: "e-mail", tone: "cordial", context: "" } };
      await vi.advanceTimersByTimeAsync(1000);
      expect(field.value).toContain("Présenter un devis");
      expect(app.view.state.doc.textContent).toBe("Note");
      expect(document.activeElement).toBe(focused);
      snapshot = { ...snapshot, mode: "manual" };
      await vi.advanceTimersByTimeAsync(1000);
      expect(field.value).toContain("Présenter un devis");
      snapshot = { ...snapshot, noteId: 2, manualPeriod: 2, draftingIntention: null };
      await vi.advanceTimersByTimeAsync(1000);
      expect(field.value).toBe("");
    } finally { app.destroy(); vi.useRealTimers(); }
  });

});
