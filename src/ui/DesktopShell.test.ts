// @vitest-environment jsdom
import { describe, it, expect, vi } from "vitest";
import { mountDesktopShell, type Snapshot } from "./DesktopShell";
import type { Proposal } from "./contracts";

const initial: Snapshot = { noteId:1, manualPeriod:1, revision:0, generation:1, mode:"manual",
  dictationMode:"content", context:"", instructions:"", dictationTransitionPending:false, inputGeneration:1,
  source:"Note", dirty:false, message:"Prêt", blocks:[{from:0,to:4,raw:"Note",kind:"paragraph"}] };

describe("fenêtre de note", () => {
  it("présente une fenêtre desktop compacte avec deux barres et un éditeur dominant", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const app = await mountDesktopShell(root, async () => initial);
    try {
      expect(root.hasAttribute("data-desktop-shell")).toBe(true);
      const toolbars = [...root.querySelectorAll<HTMLElement>('[role="toolbar"]')];
      expect(toolbars.map(toolbar => toolbar.getAttribute("aria-label"))).toEqual([
        "Modes et reformulation", "Mise en forme",
      ]);
      const context = root.querySelector('[data-workspace="context"]')!;
      const content = root.querySelector('[data-workspace="content"]')!;
      expect(context.compareDocumentPosition(content) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      expect(root.querySelector('[aria-label="État de l’écoute"]')).not.toBeNull();
    } finally { app.destroy(); }
  });
  it("réunit menus, bandeau et une note sans cible de traitement inventée", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const invoke = vi.fn(async (_command: string, _args?: Record<string, unknown>) => initial);
    const app = await mountDesktopShell(root, invoke);
    try {
      expect([...root.querySelectorAll(".menu-trigger")].map(e => e.textContent)).toEqual(["Fichier","Édition","Aide"]);
      expect(root.querySelectorAll('[role="toolbar"]')).toHaveLength(2);
      for (const action of ["Gras","Italique","Titre","reformat"]) expect(root.querySelector(`[data-action="${action}"]`)).not.toBeNull();
      expect(root.querySelectorAll(".ProseMirror")).toHaveLength(1);
      expect(root.querySelector('[data-target], [data-processing-zone]')).toBeNull();
      expect(root.textContent).not.toContain("T006 non recetté");
      expect(root.querySelector('[role="status"]')?.getAttribute("aria-live")).toBe("polite");
      expect(root.textContent).toContain("Microphone arrêté — aucun mode sélectionné");
      expect([...root.querySelectorAll<HTMLButtonElement>("[data-dictation-mode]")].every(button =>
        button.getAttribute("aria-pressed") === "false")).toBe(true);
      expect(invoke.mock.calls.map(call => call[0])).toEqual([
        "prototype_snapshot", "preferences_get",
      ]);
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
      stt:{provider:"openai",model:"gpt-realtime-whisper"},
      llm:{provider:"openai",model:"gpt-6-luna"}, lastNote:null,
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
  it("annonce une migration persistée une fois sans afficher de commandes vocales", async () => {
    vi.useFakeTimers();
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const migrated = { ...initial, preferencesMigrated:true, sttModel:"custom-stt", llmModel:"custom-llm" };
    const app = await mountDesktopShell(root, async command => command === "preferences_get"
      ? { stt:{model:"custom-stt"}, llm:{model:"custom-llm"} } : migrated);
    try {
      expect(root.textContent).toContain("préférences ont été migrées");
      expect(root.textContent).toContain("clé OpenAI");
      expect(root.textContent).toContain("custom-stt");
      expect(root.textContent).toContain("custom-llm");
      expect(root.textContent).toContain("OpenAI");
      expect(root.textContent).not.toMatch(/Mistral|Voxtral/);
      expect(root.querySelector('[data-consent="voice"]')).toBeNull();
      for (const action of ["start", "resume", "pause", "stop", "session-pause"])
        expect(root.querySelector(`[data-action="${action}"]`)).toBeNull();
      const dismiss = root.querySelector<HTMLButtonElement>('[data-action="dismiss-migration"]')!;
      dismiss.click();
      await vi.advanceTimersByTimeAsync(1000);
      expect(dismiss.parentElement?.hidden).toBe(true);
      expect(root.querySelector(".ProseMirror")?.getAttribute("contenteditable")).toBe("true");
    } finally { app.destroy(); vi.useRealTimers(); }
  });
  it("conserve l'édition manuelle si les préférences sont indisponibles", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const app = await mountDesktopShell(root, async command => {
      if (command === "preferences_get") throw new Error("private detail");
      return initial;
    });
    try {
      expect(root.textContent).toContain("Modèles configurés indisponibles");
      expect(root.textContent).not.toContain("private detail");
      expect(root.querySelector(".ProseMirror")?.getAttribute("contenteditable")).toBe("true");
      expect(root.querySelector('[data-action="dismiss-migration"]')?.parentElement?.hidden).toBe(true);
    } finally { app.destroy(); }
  });
  it("garde le contexte, le contenu et la mise en forme éditables pendant l'écoute dans chaque mode", async () => {
    for (const dictationMode of ["content", "context", "instructions"] as const) {
      const root = document.createElement("main"); document.body.replaceChildren(root);
      const snapshot: Snapshot = { ...initial, mode:"voice", listeningState:"listening", dictationMode };
      let manualAccepted = false;
      const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
        if (command !== "prototype_manual") return snapshot;
        const proposal = (args as { proposal: Proposal }).proposal;
        const source = [...proposal.patches].sort((a, b) => b.from - a.from).reduce((current, patch) =>
          current.slice(0, patch.from) + patch.text + current.slice(patch.to), snapshot.source);
        manualAccepted = true;
        return {
          transactionId: proposal.transactionId, accepted: true, message: "Modification acceptée.",
          snapshot: { ...snapshot, revision: snapshot.revision + 1, source, dirty: true,
            blocks: [{ from: 0, to: new TextEncoder().encode(source).length, raw: source, kind: "paragraph" }] },
        };
      });
      const app = await mountDesktopShell(root, invoke);
      try {
        expect(root.querySelector('[data-dictation-field="context"]')).not.toBeNull();
        expect(root.querySelector<HTMLTextAreaElement>('[data-dictation-field="context"]')?.disabled).toBe(false);
        expect(root.querySelector(".ProseMirror")?.getAttribute("contenteditable")).toBe("true");
        for (const action of ["Gras","Italique","Titre","Paragraphe","list"])
          expect(root.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)?.disabled).toBe(false);
        app.view.dispatch(app.view.state.tr.insertText("!"));
        await vi.waitFor(() => expect(manualAccepted).toBe(true));
      } finally { app.destroy(); }
    }
  });
  it("attend le choix d'un mode, puis change la cible sans commande vocale et conserve le reformulage explicite", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const actions: Array<Record<string, string | boolean>> = [];
    let active: Snapshot = initial;
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_snapshot") return initial;
      if (command === "preferences_get") return {
        stt: { provider: "openai", model: "gpt-realtime-whisper" },
        llm: { provider: "openai", model: "gpt-6-luna" }, lastNote: null,
      };
      if (command === "prototype_action") {
        const action = args?.action as Record<string, string | boolean>;
        actions.push(action);
        if (action.type === "set_dictation_mode") active = {
          ...active, mode: "voice", listeningState: "starting",
          dictationMode: action.mode as Snapshot["dictationMode"],
        };
        return active;
      }
      return active;
    });
    const app = await mountDesktopShell(root, invoke);
    try {
      expect(actions).toEqual([]);
      expect(root.querySelector<HTMLButtonElement>('[data-dictation-mode="content"]')?.getAttribute("aria-pressed")).toBe("false");
      expect(root.textContent).toContain("Microphone arrêté — aucun mode sélectionné");
      expect(root.querySelector('[data-consent="voice"]')).toBeNull();
      for (const action of ["start", "resume", "pause", "stop", "session-pause"])
        expect(root.querySelector(`[data-action="${action}"]`)).toBeNull();
      root.querySelector<HTMLButtonElement>('[data-dictation-mode="content"]')!.click();
      await vi.waitFor(() => expect(actions[0]).toEqual({ type: "set_dictation_mode", mode: "content" }));
      expect(root.querySelector<HTMLButtonElement>('[data-dictation-mode="content"]')?.getAttribute("aria-pressed")).toBe("true");
      root.querySelector<HTMLButtonElement>('[data-action="dictation-context"]')!.click();
      await vi.waitFor(() => expect(actions[1]).toEqual({ type: "set_dictation_mode", mode: "context" }));
      expect(invoke.mock.calls.some(call => call[0] === "prototype_reformat")).toBe(false);
    } finally { app.destroy(); }
  });
  it("arrête et désélectionne sur un second clic, ou bascule vers un autre mode", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let active = initial;
    const actions: Array<Record<string, string | boolean>> = [];
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_snapshot") return initial;
      if (command === "preferences_get") return {
        stt: { provider: "openai", model: "gpt-realtime-whisper" },
        llm: { provider: "openai", model: "gpt-6-luna" }, lastNote: null,
      };
      if (command !== "prototype_action") return active;
      const action = args?.action as Record<string, string | boolean>;
      if (action.type === "set_dictation_mode") {
        actions.push(action);
        const mode = action.mode as Snapshot["dictationMode"];
        active = active.mode === "voice" && active.dictationMode === mode
          ? { ...active, mode: "manual", listeningState: "stopped", dictationMode: "content", instructions: "" }
          : { ...active, mode: "voice", listeningState: "starting", dictationMode: mode,
            instructions: mode === "instructions" ? "Consigne temporaire." : active.instructions };
      }
      return active;
    });
    const app = await mountDesktopShell(root, invoke);
    const clickMode = (mode: Snapshot["dictationMode"]) =>
      root.querySelector<HTMLButtonElement>(`[data-dictation-mode="${mode}"]`)!.click();
    const selected = (mode: Snapshot["dictationMode"]) =>
      root.querySelector<HTMLButtonElement>(`[data-dictation-mode="${mode}"]`)!.getAttribute("aria-pressed");
    try {
      clickMode("content");
      await vi.waitFor(() => expect(selected("content")).toBe("true"));
      clickMode("content");
      await vi.waitFor(() => expect(root.textContent).toContain("aucun mode sélectionné"));
      expect(["content", "context", "instructions"].every(mode =>
        root.querySelector(`[data-dictation-mode="${mode}"]`)?.getAttribute("aria-pressed") === "false")).toBe(true);

      clickMode("context");
      await vi.waitFor(() => expect(selected("context")).toBe("true"));
      clickMode("instructions");
      await vi.waitFor(() => expect(selected("instructions")).toBe("true"));
      const popup = root.querySelector<HTMLElement>(".instruction-popup")!;
      expect(popup.hidden).toBe(false);
      clickMode("instructions");
      await vi.waitFor(() => expect(popup.hidden).toBe(true));
      expect(root.querySelector<HTMLTextAreaElement>('[data-dictation-field="instructions"]')?.value).toBe("");
      expect(actions.map(action => action.mode)).toEqual([
        "content", "content", "context", "instructions", "instructions",
      ]);
    } finally { app.destroy(); }
  });
  it("affiche les icônes accessibles et ne ferme la popup Instruction qu'après la transition", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let active: Snapshot = initial;
    let finishTransition!: (snapshot: Snapshot) => void;
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_snapshot") return initial;
      if (command !== "prototype_action") return active;
      const action = args?.action as Record<string, string | boolean>;
      if (action.type === "set_dictation_mode" && action.mode === "context") {
        return new Promise<Snapshot>(resolve => { finishTransition = resolve; });
      }
      if (action.type === "set_dictation_mode") active = {
        ...active, mode: "voice", listeningState: "listening",
        dictationMode: action.mode as Snapshot["dictationMode"],
      };
      if (action.type === "set_instructions") active = { ...active, instructions: String(action.text) };
      return active;
    });
    const app = await mountDesktopShell(root, invoke);
    try {
      for (const mode of ["content", "context", "instructions"]) {
        const control = root.querySelector<HTMLButtonElement>(`[data-dictation-mode="${mode}"]`)!;
        expect(control.querySelector(".mode-icon")).not.toBeNull();
        expect(control.getAttribute("aria-label")).toBeTruthy();
        expect(control.title).toBeTruthy();
      }
      for (const action of ["Gras", "Italique", "Titre", "Paragraphe", "list"]) {
        const control = root.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)!;
        expect(control.querySelector(".format-icon")).not.toBeNull();
        expect(control.getAttribute("aria-label")).toBeTruthy();
        expect(control.title).toBeTruthy();
      }
      const popup = root.querySelector<HTMLElement>(".instruction-popup")!;
      expect(popup.hidden).toBe(true);
      root.querySelector<HTMLButtonElement>('[data-dictation-mode="instructions"]')!.click();
      await vi.waitFor(() => expect(popup.hidden).toBe(false));
      const field = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="instructions"]')!;
      field.value = "Style concis.";
      field.dispatchEvent(new InputEvent("input", { bubbles: true }));
      await vi.waitFor(() => expect(active.instructions).toBe("Style concis."));

      root.querySelector<HTMLButtonElement>('[data-dictation-mode="context"]')!.click();
      await vi.waitFor(() => expect(finishTransition).toBeTypeOf("function"));
      expect(popup.hidden).toBe(false);
      finishTransition({ ...active, dictationMode: "context", instructions: "" });
      await vi.waitFor(() => expect(popup.hidden).toBe(true));
      expect(field.value).toBe("");
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
  it("place le contexte au-dessus de la note sans exposer les métadonnées legacy", async () => {
    vi.useFakeTimers();
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let snapshot: Snapshot = { ...initial, mode: "voice", context: "Contexte initial", draftingIntention: null };
    const app = await mountDesktopShell(root, async () => snapshot);
    try {
      const field = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="context"]')!;
      const editor = root.querySelector('[aria-label="Note"]')!;
      expect(field.compareDocumentPosition(editor) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      const focused = root.querySelector<HTMLButtonElement>(".menu-trigger")!; focused.focus();
      snapshot = { ...snapshot, context: "Contexte actualisé",
        draftingIntention: { purpose: "Présenter un devis", form: "e-mail", tone: "cordial", context: "" } };
      await vi.advanceTimersByTimeAsync(1000);
      expect(field.value).toBe("Contexte actualisé");
      expect(root.textContent).not.toContain("Présenter un devis");
      expect(app.view.state.doc.textContent).toBe("Note");
      expect(document.activeElement).toBe(focused);
      snapshot = { ...snapshot, mode: "manual" };
      await vi.advanceTimersByTimeAsync(1000);
      expect(field.value).toBe("Contexte actualisé");
      snapshot = { ...snapshot, noteId: 2, manualPeriod: 2, context: "", draftingIntention: null };
      await vi.advanceTimersByTimeAsync(1000);
      expect(field.value).toBe("");
    } finally { app.destroy(); vi.useRealTimers(); }
  });
  it("n'appelle pas le LLM à l'édition ou au changement de mode, seulement sur demande", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const formatted: Snapshot = {
      ...initial, revision: 1, source: "# Note reformulée",
      blocks: [{ from: 0, to: 18, raw: "# Note reformulée", kind: "heading" }],
    };
    const invoke = vi.fn(async (command: string) => command === "prototype_reformat" ? formatted : initial);
    const app = await mountDesktopShell(root, invoke);
    try {
      root.querySelector<HTMLButtonElement>('[data-action="dictation-context"]')!.click();
      await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("prototype_action",
        expect.objectContaining({ action: expect.objectContaining({ type: "set_dictation_mode" }) })));
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(0);

      root.querySelector<HTMLButtonElement>('[data-action="reformat"]')!.click();
      await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("prototype_reformat", {
        noteId: initial.noteId, baseRevision: initial.revision,
      }));
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(1);
      expect(app.view.state.doc.textContent).toBe("Note reformulée");
    } finally { app.destroy(); }
  });
  it("reformule depuis la popup après avoir enregistré le contexte et les instructions courants", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let active: Snapshot = initial;
    let sentFields: { context: string; instructions: string } | undefined;
    const formatted: Snapshot = {
      ...initial, revision: 1, source: "Note reformulée",
      blocks: [{ from: 0, to: 16, raw: "Note reformulée", kind: "paragraph" }],
    };
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_snapshot") return active;
      if (command === "prototype_action") {
        const action = args?.action as Record<string, string | boolean>;
        if (action.type === "set_dictation_mode") active = {
          ...active, mode: "voice", dictationMode: action.mode as Snapshot["dictationMode"],
          instructions: action.mode === "instructions" ? active.instructions : "",
        };
        if (action.type === "set_context") active = { ...active, context: String(action.text) };
        if (action.type === "set_instructions") active = { ...active, instructions: String(action.text) };
        return active;
      }
      if (command === "prototype_reformat") {
        sentFields = { context: active.context, instructions: active.instructions };
        return formatted;
      }
      return active;
    });
    const app = await mountDesktopShell(root, invoke);
    try {
      const popup = root.querySelector<HTMLElement>(".instruction-popup")!;
      root.querySelector<HTMLButtonElement>('[data-dictation-mode="instructions"]')!.click();
      await vi.waitFor(() => expect(popup.hidden).toBe(false));
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(0);

      const context = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="context"]')!;
      const instructions = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="instructions"]')!;
      context.value = "Contexte courant";
      context.dispatchEvent(new InputEvent("input", { bubbles: true }));
      instructions.value = "Rester concis";
      instructions.dispatchEvent(new InputEvent("input", { bubbles: true }));
      root.querySelector<HTMLButtonElement>('[data-action="instruction-reformat"]')!.click();

      await vi.waitFor(() => expect(invoke.mock.calls.some(call => call[0] === "prototype_reformat")).toBe(true));
      await vi.waitFor(() => expect(sentFields).toEqual({
        context: "Contexte courant", instructions: "Rester concis",
      }));
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_action")
        .map(call => call[1]?.action)).toContainEqual({ type: "set_instructions", text: "Rester concis" });
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(1);
      expect(app.view.state.doc.textContent).toBe("Note reformulée");

      root.querySelector<HTMLButtonElement>('[data-dictation-mode="context"]')!.click();
      await vi.waitFor(() => expect(popup.hidden).toBe(true));
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(1);
    } finally { app.destroy(); }
  });
  it("n'envoie pas de reformulation si le contexte courant n'a pas été enregistré", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let active: Snapshot = initial;
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_action") {
        const action = args?.action as Record<string, string | boolean>;
        if (action.type === "set_context") throw "synthetic save failure";
        if (action.type === "set_dictation_mode") active = {
          ...active, mode: "voice", dictationMode: action.mode as Snapshot["dictationMode"],
        };
        return active;
      }
      if (command === "prototype_snapshot") return active;
      return active;
    });
    const app = await mountDesktopShell(root, invoke);
    try {
      const popup = root.querySelector<HTMLElement>(".instruction-popup")!;
      root.querySelector<HTMLButtonElement>('[data-dictation-mode="instructions"]')!.click();
      await vi.waitFor(() => expect(popup.hidden).toBe(false));
      const context = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="context"]')!;
      context.value = "Contexte non enregistré";
      context.dispatchEvent(new InputEvent("input", { bubbles: true }));
      root.querySelector<HTMLButtonElement>('[data-action="instruction-reformat"]')!.click();

      await vi.waitFor(() => expect(root.textContent).toContain(
        "Contexte ou instructions non enregistrés ; aucune reformulation lancée.",
      ));
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(0);
    } finally { app.destroy(); }
  });
  it("désactive le déclenchement dupliqué pendant la reformulation", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    let finish!: (snapshot: Snapshot) => void;
    let active: Snapshot = initial;
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_reformat") return new Promise<Snapshot>(resolve => { finish = resolve; });
      if (command === "prototype_action") {
        const action = args?.action as Record<string, string | boolean>;
        if (action.type === "set_dictation_mode") active = {
          ...active, mode: "voice", dictationMode: action.mode as Snapshot["dictationMode"],
        };
      }
      return active;
    });
    const app = await mountDesktopShell(root, invoke);
    root.querySelector<HTMLButtonElement>('[data-dictation-mode="instructions"]')!.click();
    const reformat = root.querySelector<HTMLButtonElement>('[data-action="instruction-reformat"]')!;
    try {
      await vi.waitFor(() => expect(root.querySelector<HTMLElement>(".instruction-popup")?.hidden).toBe(false));
      reformat.click();
      await vi.waitFor(() => expect(reformat.disabled).toBe(true));
      expect(root.querySelector<HTMLButtonElement>('[data-action="reformat"]')!.disabled).toBe(true);
      expect(root.querySelector('[data-reformat-status]')?.textContent).toContain("en cours");
      await vi.waitFor(() => expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(1));
      reformat.click();
      expect(invoke.mock.calls.filter(call => call[0] === "prototype_reformat")).toHaveLength(1);
      finish({ ...initial, revision: 1, source: "Texte reformulé",
        blocks: [{ from: 0, to: 15, raw: "Texte reformulé", kind: "paragraph" }] });
      await vi.waitFor(() => expect(reformat.disabled).toBe(false));
      expect(app.view.state.doc.textContent).toBe("Texte reformulé");
    } finally { app.destroy(); }
  });
  it("préserve une note plus récente et masque le détail d'un échec périmé", async () => {
    const root = document.createElement("main"); document.body.replaceChildren(root);
    const latest: Snapshot = {
      ...initial, revision: 1, source: "Modification récente",
      message: "Résultat de reformulation périmé ; note inchangée.",
      blocks: [{ from: 0, to: 20, raw: "Modification récente", kind: "paragraph" }],
    };
    let snapshot = initial;
    const invoke = vi.fn(async (command: string) => {
      if (command === "prototype_reformat") {
        snapshot = latest;
        throw "private stale response";
      }
      if (command === "prototype_snapshot") return snapshot;
      return initial;
    });
    const app = await mountDesktopShell(root, invoke);
    try {
      const button = root.querySelector<HTMLButtonElement>('[data-action="reformat"]')!;
      button.click();
      await vi.waitFor(() => expect(button.disabled).toBe(true));
      await vi.waitFor(() => expect(button.disabled).toBe(false));
      await vi.waitFor(() => expect(app.view.state.doc.textContent).toBe("Modification récente"));
      expect(root.textContent).toContain("Reformulation impossible ; contenu conservé.");
      expect(root.textContent).not.toContain("private stale response");
    } finally { app.destroy(); }
  });

});
