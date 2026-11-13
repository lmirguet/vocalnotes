// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { mountPrototype, type Snapshot } from "./Prototype";
import { readFileSync } from "node:fs";
import { URL as NodeURL } from "node:url";

const styles = readFileSync(new NodeURL("../style.css", import.meta.url), "utf8");

const initial: Snapshot = {
  noteId: 1, manualPeriod: 1, revision: 0, generation: 1, mode: "manual",
  dictationMode: "content", context: "", instructions: "", dictationTransitionPending: false, inputGeneration: 1,
  source: "bonjour\n\n```rs\nopaque()\n```\n", dirty: false,
  message: "Prototype partiel.", blocks: [
    { from: 0, to: 7, raw: "bonjour", kind: "paragraph" },
    { from: 9, to: 27, raw: "```rs\nopaque()\n```", kind: "opaque" },
  ],
};
const preferences = {
  stt: { provider: "openai", model: "gpt-realtime-whisper" },
  llm: { provider: "openai", model: "gpt-6-luna" }, lastNote: null,
};

async function setup(invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown> = vi.fn(async (command: string) => command === "preferences_get" ? preferences : initial)) {
  const root = document.createElement("main");
  document.body.replaceChildren(root);
  const app = await mountPrototype(root, invoke);
  return { root, invoke, app };
}

describe("prototype natif partiel", () => {
  it("permet l'édition du contexte en permanence et masque les instructions hors de leur mode", async () => {
    let snapshot = initial;
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_action") {
        const action = args!.action as { type: string; mode?: Snapshot["dictationMode"]; text?: string };
        if (action.type === "set_dictation_mode") snapshot = {
          ...snapshot, mode: "voice", dictationMode: action.mode!,
        };
        if (action.type === "set_context") snapshot = { ...snapshot, context: action.text! };
        if (action.type === "set_instructions") snapshot = { ...snapshot, instructions: action.text! };
      }
      return command === "preferences_get" ? preferences : snapshot;
    });
    const { root, app } = await setup(invoke);
    try {
      const context = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="context"]')!;
      const editor = root.querySelector<HTMLElement>(".ProseMirror")!;
      expect(context).not.toBeNull();
      expect(editor).not.toBeNull();
      expect(root.querySelector<HTMLTextAreaElement>('[data-dictation-field="instructions"]')
        ?.closest<HTMLElement>(".instruction-popup")?.hidden).toBe(true);

      context.value = "Réunion du lundi.";
      context.dispatchEvent(new InputEvent("input", { bubbles: true }));
      await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("prototype_action", {
        action: { type: "set_context", text: "Réunion du lundi." },
      }));

      root.querySelector<HTMLButtonElement>('[data-dictation-mode="instructions"]')!.click();
      await vi.waitFor(() => expect(root.querySelector<HTMLTextAreaElement>('[data-dictation-field="instructions"]')
        ?.closest<HTMLElement>(".instruction-popup")?.hidden).toBe(false));
      const instructions = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="instructions"]')!;
      instructions.value = "Style concis.";
      instructions.dispatchEvent(new InputEvent("input", { bubbles: true }));
      await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("prototype_action", {
        action: { type: "set_instructions", text: "Style concis." },
      }));
      expect(root.querySelector(".ProseMirror")).toBe(editor);
    } finally { app.destroy(); }
  });

  it("sélectionne explicitement les trois destinations et expose le mode sélectionné", async () => {
    let snapshot = initial;
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_action") {
        const action = args!.action as { type: string; mode: Snapshot["dictationMode"] };
        snapshot = { ...snapshot, mode: "voice", dictationMode: action.mode };
      }
      return command === "preferences_get" ? preferences : snapshot;
    });
    const { root, app } = await setup(invoke);
    try {
      const modes = ["content", "context", "instructions"] as const;
      for (const mode of modes) {
        const control = root.querySelector<HTMLButtonElement>(`[data-dictation-mode="${mode}"]`)!;
        control.click();
        await vi.waitFor(() => expect(control.getAttribute("aria-pressed")).toBe("true"));
        expect(invoke).toHaveBeenCalledWith("prototype_action", {
          action: { type: "set_dictation_mode", mode },
        });
      }
    } finally { app.destroy(); }
  });

  it("garde Instructions sélectionné pendant la finalisation d'un segment en attente", async () => {
    let snapshot = {
      ...initial, mode: "voice" as const, listeningState: "listening" as const,
      dictationMode: "instructions" as const, instructions: "Style concis.",
    };
    const invoke = vi.fn(async (command: string) => {
      if (command === "prototype_action") snapshot = {
        ...snapshot, mode: "voice", dictationTransitionPending: true,
      };
      return command === "preferences_get" ? preferences : snapshot;
    });
    const { root, app } = await setup(invoke);
    try {
      root.querySelector<HTMLButtonElement>('[data-dictation-mode="context"]')!.click();
      await vi.waitFor(() => expect(root.querySelector('[data-dictation-mode="instructions"]')?.getAttribute("aria-pressed")).toBe("true"));
      expect(root.textContent).toContain("transcription en attente");
    } finally { app.destroy(); }
  });

  it("actualise un avertissement disque sans révision ni remplacement du texte", async () => {
    vi.useFakeTimers();
    let calls = 0;
    const invoke = vi.fn(async () => ++calls === 1 ? initial : {
      ...initial, storageMessage: "Modification externe détectée ; contenu conservé.",
    });
    const { root, app } = await setup(invoke);
    try {
      const editor = root.querySelector(".ProseMirror");
      const content = editor?.textContent;
      await vi.advanceTimersByTimeAsync(1000);
      expect(root.querySelector('[role="status"]')?.textContent).toContain("Modification externe détectée");
      expect(root.querySelector(".ProseMirror")).toBe(editor);
      expect(editor?.textContent).toBe(content);
    } finally { app.destroy(); vi.useRealTimers(); }
  });
  it("conserve le bilan vocal après arrêt sans inventer de texte appliqué", async () => {
    const { root, app } = await setup(vi.fn(async () => ({
      ...initial, message: "Arrêt explicite ; passages en attente non intégrés.",
      voiceApplied: 0, voiceClarifications: 2, voiceRejected: 1, voiceNoChange: 3,
      lastVoiceResult: "Précision demandée ; aucune action appliquée.",
    })));
    expect(root.querySelector('[role="status"]')?.textContent).toContain(
      "Bilan vocal : 0 modification(s), 2 clarification(s), 1 refus, 3 sans modification");
    expect(root.querySelector('[role="status"]')?.textContent).toContain(
      "Précision demandée ; aucune action appliquée.");
    app.destroy();
  });
  it("affiche menus et éditeur sans démarrer de mode automatiquement", async () => {
    const { root, invoke, app } = await setup();
    const nav = root.querySelector('nav[aria-label="Menus"]')!;
    const triggers = [...nav.querySelectorAll<HTMLButtonElement>("button[aria-expanded]")];
    expect(triggers.map(b => b.textContent)).toEqual(["Fichier", "Édition", "Aide"]);
    expect(nav.querySelector("details")).toBeNull();
    expect(triggers.every(b => b.getAttribute("aria-expanded") === "false")).toBe(true);
    for (const trigger of triggers) {
      expect(document.getElementById(trigger.getAttribute("aria-controls")!)?.hidden).toBe(true);
    }
    expect(nav.contains(root.querySelector('[role="toolbar"]'))).toBe(false);
    for (const action of ["new", "open", "save", "save-current", "close", "cancel-close", "copy", "undo", "redo"]) {
      expect(nav.querySelectorAll(`[data-action="${action}"]`)).toHaveLength(1);
    }
    expect(root.querySelector(".ProseMirror")).not.toBeNull();
    expect(root.querySelector("[data-opaque]")?.textContent).toContain("opaque()");
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ["prototype_snapshot", undefined], ["preferences_get", undefined],
    ]);
    expect(root.textContent).toContain("Microphone arrêté — aucun mode sélectionné");
    expect(root.querySelector<HTMLButtonElement>('[data-dictation-mode="content"]')?.getAttribute("aria-pressed")).toBe("false");
    expect(root.querySelector('[data-consent="voice"]')).toBeNull();
    expect(root.querySelector('[data-action="start"]')).toBeNull();
    app.destroy();
  });

  it("déroule un seul panneau, referme par bouton, action, clic extérieur et Escape avec focus cohérent", async () => {
    const { root, app, invoke } = await setup();
    const [files, edits, help] = [...root.querySelectorAll<HTMLButtonElement>("nav button[aria-expanded]")];
    const panel = (b: HTMLButtonElement) => document.getElementById(b.getAttribute("aria-controls")!)!;
    files!.click();
    expect(panel(files!).hidden).toBe(false);
    edits!.click();
    expect(panel(files!).hidden).toBe(true);
    expect(panel(edits!).hidden).toBe(false);
    edits!.click();
    expect(panel(edits!).hidden).toBe(true);
    files!.click();
    const path = root.querySelector<HTMLInputElement>('[aria-label="Destination Markdown locale"]')!;
    path.focus();
    path.value = "/synthetic.md";
    path.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(panel(files!).hidden).toBe(true);
    expect(document.activeElement).toBe(files);
    files!.click();
    root.querySelector<HTMLButtonElement>('[data-action="open"]')!.click();
    expect(panel(files!).hidden).toBe(true);
    expect(document.activeElement).toBe(files);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("prototype_action", {
      action: { type: "open", path: "/synthetic.md" },
    }));
    help!.click();
    root.querySelector('[role="status"]')!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(panel(help!).hidden).toBe(true);
    expect(path.value).toBe("/synthetic.md");
    app.destroy();
  });

  it("navigation disclosure : flèches/Home/End sur les intitulés, Tab natif dans les champs, sortie et nettoyage", async () => {
    const { root, app } = await setup();
    const triggers = [...root.querySelectorAll<HTMLButtonElement>("nav button[aria-expanded]")];
    const key = (b: HTMLElement, value: string) => b.dispatchEvent(new KeyboardEvent("keydown", { key: value, bubbles: true, cancelable: true }));
    triggers[0]!.focus();
    key(triggers[0]!, "ArrowRight");
    expect(document.activeElement).toBe(triggers[1]);
    key(triggers[1]!, "End");
    expect(document.activeElement).toBe(triggers[2]);
    key(triggers[2]!, "Home");
    expect(document.activeElement).toBe(triggers[0]);
    key(triggers[0]!, "ArrowLeft");
    expect(document.activeElement).toBe(triggers[2]);
    triggers[0]!.click();
    const path = root.querySelector<HTMLInputElement>('[aria-label="Destination Markdown locale"]')!;
    path.focus();
    expect(key(path, "ArrowRight")).toBe(true); // text caret remains native
    expect(key(path, "Tab")).toBe(true); // no trap or menuitem semantics
    expect(document.activeElement).toBe(path);
    const outside = root.querySelector<HTMLTextAreaElement>('[data-dictation-field="context"]')!;
    outside.focus();
    expect(triggers[0]!.getAttribute("aria-expanded")).toBe("false");
    expect(document.activeElement).toBe(outside);
    app.destroy();
    triggers[0]!.click();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(document.activeElement).toBe(outside);
  });

  it("applique le shell compact, les icônes et la popup d'instruction sans bordure", async () => {
    const { root, app } = await setup(async command =>
      command === "prototype_action"
        ? { ...initial, mode: "voice", dictationMode: "instructions" }
        : initial);
    const style = document.createElement("style");
    style.textContent = styles;
    document.head.append(style);
    try {
      const nav = root.querySelector("nav")!;
      const trigger = nav.querySelector<HTMLButtonElement>("button[aria-expanded]")!;
      const panel = document.getElementById(trigger.getAttribute("aria-controls")!)!;
      expect(getComputedStyle(nav).display).toBe("flex");
      expect(getComputedStyle(nav).flexWrap).toBe("nowrap");
      expect(getComputedStyle(panel).display).toBe("none");
      trigger.click();
      expect(getComputedStyle(panel).position).toBe("absolute");
      expect(getComputedStyle(panel).display).not.toBe("none");
      expect(getComputedStyle(panel).zIndex).toBe("10");
      root.querySelector<HTMLButtonElement>('[data-dictation-mode="instructions"]')!.click();
      const popup = root.querySelector<HTMLElement>(".instruction-popup")!;
      await vi.waitFor(() => expect(popup.hidden).toBe(false));
      expect(getComputedStyle(popup).position).toBe("absolute");
      expect(getComputedStyle(popup).borderTopWidth).toBe("0px");
      expect(getComputedStyle(popup).zIndex).toBe("20");
      expect(getComputedStyle(root.querySelector(".mode-icon")!).width).toBe("16px");
    } finally { style.remove(); app.destroy(); }
  });

  it("garde un conflit de sauvegarde visible après changement du message de session", async () => {
    const { root, app } = await setup(vi.fn(async () => ({
      ...initial, mode: "paused", dirty: true, message: "Pause — micro arrêté.",
      storageMessage: "Conflit disque. Version locale : /tmp/local.md ; disque : /tmp/externe.md",
    })));
    const status = root.querySelector('[role="status"]')!;
    expect(status.textContent).toContain("Pause");
    expect(status.textContent).toContain("Conflit disque");
    expect(status.textContent).toContain("/tmp/local.md");
    expect(status.textContent).toContain("/tmp/externe.md");
    app.destroy();
  });

  it("champ masqué vidé avant attente IPC, aucune valeur renvoyée au store", async () => {
    const invoke = vi.fn(async (command: string) => command === "prototype_snapshot" ? initial : undefined);
    const { root, app } = await setup(invoke);
    const input = root.querySelector<HTMLInputElement>('input[type="password"]')!;
    input.value = "synthetic-test-secret";
    root.querySelector<HTMLButtonElement>('[data-action="key-save"]')!.click();
    expect(input.value).toBe("");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("prototype_set_key", { secret: "synthetic-test-secret" }));
    expect(root.textContent).not.toContain("synthetic-test-secret");
    app.destroy();
  });

  it("erreur IPC expurgée et sans prétendre une sauvegarde réussie", async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === "prototype_snapshot") return initial;
      throw new Error("sensitive provider payload");
    });
    const { root, app } = await setup(invoke);
    root.querySelector<HTMLButtonElement>('[data-action="key-delete"]')!.click();
    await vi.waitFor(() => expect(root.textContent).toContain("Échec"));
    expect(root.textContent).not.toContain("sensitive provider payload");
    app.destroy();
  });

  it("pas de ressource distante ou HTML actif à partir de la note", async () => {
    const s = { ...initial, blocks: [{ from: 0, to: 30, kind: "opaque", raw: '<img src="https://invalid/x">' }] };
    const { root, app } = await setup(vi.fn(async () => s));
    expect(root.querySelector("img")).toBeNull();
    expect(root.querySelector("[data-opaque]")?.textContent).toContain("<img");
    app.destroy();
  });

  it("garde l'édition disponible pendant la capture sans afficher de commandes vocales", async () => {
    const { root, app, invoke } = await setup(vi.fn(async () => ({ ...initial, mode: "voice" })));
    // Locate by label, not DOM text presence alone.
    const trigger = [...root.querySelectorAll<HTMLButtonElement>("nav button[aria-expanded]")]
      .find(b => b.textContent === "Édition")!;
    trigger.click();
    expect(root.querySelector(".ProseMirror")?.getAttribute("contenteditable")).toBe("true");
    expect(root.querySelector<HTMLButtonElement>('[data-action="undo"]')?.disabled).toBe(true);
    expect(root.querySelector<HTMLButtonElement>('[data-action="redo"]')?.disabled).toBe(true);
    root.querySelector<HTMLButtonElement>('[data-action="undo"]')!.click();
    expect(trigger.getAttribute("aria-expanded")).toBe("true");
    expect(vi.mocked(invoke).mock.calls).toEqual([
      ["prototype_snapshot", undefined], ["preferences_get", undefined],
    ]);
    expect(root.querySelector('[data-action="start"]')).toBeNull();
    app.destroy();
  });

  it("sérialise deux propositions, sauvegarde bloquée tant que le cœur n'a pas acquitté", async () => {
    const simple = { ...initial, source: "a", blocks: [{ from: 0, to: 1, raw: "a", kind: "paragraph" }] };
    const pending: Array<(ack: unknown) => void> = [];
    const proposals: Array<{ transactionId: string; baseRevision: number }> = [];
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_snapshot") return simple;
      if (command === "prototype_manual") {
        proposals.push(args!.proposal as typeof proposals[number]);
        return new Promise(resolve => pending.push(resolve));
      }
      return simple;
    });
    const { app, root } = await setup(invoke);
    app.view.dispatch(app.view.state.tr.insertText("b", 2));
    app.view.dispatch(app.view.state.tr.insertText("c", 3));
    expect(proposals).toHaveLength(1);
    expect(root.textContent).toContain("projection en attente");
    expect(root.querySelector<HTMLButtonElement>('[data-action="undo"]')?.disabled).toBe(true);
    expect(root.querySelector('[data-action="start"]')).toBeNull();
    root.querySelector<HTMLButtonElement>('[data-action="save"]')!.click();
    root.querySelector<HTMLButtonElement>('[data-action="close"]')!.click();
    expect(invoke.mock.calls.filter(call => call[0] === "prototype_action"
      && (call[1] as { action?: { type?: string } } | undefined)?.action?.type !== "set_dictation_mode"))
      .toHaveLength(0);
    expect(invoke.mock.calls.some(call => call[0] === "prototype_finish_close")).toBe(false);
    pending[0]!({ accepted: true, transactionId: proposals[0]!.transactionId, message: "OK",
      snapshot: { ...simple, revision: 1, source: "ab", dirty: true, blocks: [{ from: 0, to: 2, raw: "ab", kind: "paragraph" }] } });
    await vi.waitFor(() => expect(proposals).toHaveLength(2));
    expect(proposals[1]!.baseRevision).toBe(1);
    pending[1]!({ accepted: true, transactionId: proposals[1]!.transactionId, message: "OK",
      snapshot: { ...simple, revision: 2, source: "abc", dirty: true, blocks: [{ from: 0, to: 3, raw: "abc", kind: "paragraph" }] } });
    await vi.waitFor(() => expect(root.textContent).not.toContain("projection en attente"));
    expect(app.view.state.doc.textContent).toBe("abc");
    expect(root.querySelector<HTMLButtonElement>('[data-action="undo"]')?.disabled).toBe(false);
    app.destroy();
  });

  it("reconstruit sur refus sans rejeu et garde la saisie séparée en mémoire", async () => {
    const simple = { ...initial, source: "a", blocks: [{ from: 0, to: 1, raw: "a", kind: "paragraph" }] };
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_snapshot") return simple;
      if (command === "prototype_manual") return { accepted: false,
        transactionId: (args!.proposal as { transactionId: string }).transactionId, snapshot: simple };
      return simple;
    });
    const { app, root } = await setup(invoke);
    app.view.dispatch(app.view.state.tr.insertText("b", 2));
    await vi.waitFor(() => expect(root.textContent).toContain("aucun rejeu"));
    expect(app.view.state.doc.textContent).toBe("a");
    expect(root.querySelector('[aria-label="Saisie non acceptée récupérable en mémoire"]')?.textContent).toBe("ab");
    expect(root.querySelector<HTMLButtonElement>('[data-action="undo"]')?.disabled).toBe(true);
    expect(invoke.mock.calls.filter(call => call[0] === "prototype_manual")).toHaveLength(1);
    app.destroy();
  });

  it("ne demande pas de consentement ni de démarrage au lancement", async () => {
    const invoke = vi.fn(async (command: string, _args?: Record<string, unknown>) =>
      command === "preferences_get" ? preferences : initial);
    const { root, app } = await setup(invoke);
    expect(root.querySelector('[data-consent="voice"]')).toBeNull();
    expect(root.querySelector('[data-action="start"]')).toBeNull();
    expect(root.querySelector('[data-action="pause"]')).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("prototype_action", expect.anything());
    expect(root.textContent).toContain("Microphone arrêté — aucun mode sélectionné");
    app.destroy();
  });

  it("propose un nouveau paragraphe au cœur et reconstruit sa structure canonique", async () => {
    const simple = { ...initial, source: "ab", blocks: [{ from: 0, to: 2, raw: "ab", kind: "paragraph" }] };
    const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
      if (command === "prototype_snapshot") return simple;
      const p = args!.proposal as { transactionId: string; patches: unknown[] };
      return { accepted: true, transactionId: p.transactionId, message: "OK",
        snapshot: { ...simple, revision: 1, source: "a\n\nb", blocks: [
          { from: 0, to: 1, raw: "a", kind: "paragraph" }, { from: 3, to: 4, raw: "b", kind: "paragraph" },
        ] } };
    });
    const { app } = await setup(invoke);
    app.view.dispatch(app.view.state.tr.split(2));
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("prototype_manual", expect.objectContaining({
      proposal: expect.objectContaining({ patches: [{ from: 0, to: 2, expected: "ab", text: "a\n\nb" }] }),
    })));
    await vi.waitFor(() => expect(app.view.state.doc.childCount).toBe(2));
    app.destroy();
  });

  it("suspend l'édition sur perte de synchronisation et ne rejoue aucune proposition", async () => {
    vi.useFakeTimers();
    let calls = 0;
    const invoke = vi.fn(async (command: string, _args?: Record<string, unknown>) => {
      if (command === "preferences_get") return preferences;
      if (++calls === 1) return initial;
      throw new Error("private error");
    });
    const { root, app } = await setup(invoke);
    try {
      await vi.advanceTimersByTimeAsync(1000);
      expect(root.querySelector(".ProseMirror")?.getAttribute("contenteditable")).toBe("false");
      expect(root.textContent).toContain("édition suspendue");
      expect(root.textContent).not.toContain("private error");
      expect(invoke.mock.calls).toEqual([
        ["prototype_snapshot", undefined], ["preferences_get", undefined],
        ["prototype_snapshot", undefined],
      ]);
    } finally { app.destroy(); vi.useRealTimers(); }
  });
});
