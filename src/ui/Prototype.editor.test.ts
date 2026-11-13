// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { TextSelection } from "prosemirror-state";
import { mountPrototype, type Snapshot } from "./Prototype";

const empty: Snapshot = { noteId: 1, manualPeriod: 1, revision: 0, generation: 1,
  mode: "manual", dictationMode: "content", context: "", instructions: "",
  dictationTransitionPending: false, inputGeneration: 1,
  source: "", dirty: false, message: "Manuel.",
  blocks: [{ from: 0, to: 0, raw: "", kind: "paragraph" }] };
type Proposal = { transactionId: string; baseRevision: number;
  patches: Array<{ from: number; to: number; expected: string; text: string }> };
const cleanups: Array<() => void> = [];
afterEach(() => { cleanups.splice(0).forEach(f => f()); vi.useRealTimers(); });

// Deterministic transport contract: checks actual UTF-8 preconditions/revisions,
// drops empty paragraphs like pulldown-cmark. Native file proofs are separate.
async function editor(input: string | Snapshot = "", deferred = false) {
  const source = typeof input === "string" ? input : input.source;
  let s: Snapshot = typeof input !== "string" ? input : { ...empty, source, blocks: source ? [
    { from: 0, to: new TextEncoder().encode(source).length, raw: source, kind: "paragraph" },
  ] : empty.blocks };
  const pending: Array<() => void> = [];
  let reject = false;
  const invoke = vi.fn(async (cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "prototype_snapshot") return s;
    if (cmd === "prototype_manual") {
      const p = args!.proposal as Proposal;
      return new Promise(resolve => {
        const ack = () => {
          expect(p.baseRevision).toBe(s.revision);
          if (!reject) {
            let next = s.source;
            for (const patch of [...p.patches].sort((a, b) => b.from - a.from)) {
              const b = new TextEncoder().encode(next);
              const decode = (from: number, to: number) => new TextDecoder().decode(b.slice(from, to));
              expect(decode(patch.from, patch.to)).toBe(patch.expected);
              next = decode(0, patch.from) + patch.text + decode(patch.to, b.length);
            }
            let offset = 0;
            const blocks = next.split("\n\n").flatMap(raw => {
              const from = offset; offset += new TextEncoder().encode(raw).length + 2;
              return raw ? [{ from, to: offset - 2, raw, kind: "paragraph" }] : [];
            });
            s = { ...s, source: next, blocks: blocks.length ? blocks : empty.blocks,
              revision: s.revision + 1, dirty: true };
          }
          resolve({ transactionId: p.transactionId, accepted: !reject, message: "Acquitté.", snapshot: s });
        };
        if (deferred) pending.push(ack); else ack();
      });
    }
    if (cmd === "prototype_action") {
      const a = args!.action as { type: string; path?: string };
      if (a.type === "new") {
        if (s.dirty) throw "Sauvegarder la note modifiée avant de changer de note.";
        s = { ...empty, noteId: s.noteId + 1, manualPeriod: s.manualPeriod + 1,
          message: "Nouvelle note créée — manuel, aucune capture." };
      } else if (a.type === "save_copy") {
        if (!a.path) throw "Chemin absolu vers un fichier .md requis.";
        s = { ...s, dirty: false, message: "Nouvelle copie Markdown enregistrée ; aucun fichier existant remplacé." };
      } else if (a.type === "save") {
        throw "Choisir une destination Markdown locale avant la sauvegarde ; aucun effet.";
      }
      return s;
    }
    throw new Error("unexpected command");
  });
  const root = document.createElement("main");
  document.body.replaceChildren(root);
  const app = await mountPrototype(root, invoke);
  // jsdom has no Range geometry. Bypass only scrolling, not key handling,
  // transactions, selection, focus, projection or the IPC contract assertions.
  app.view.setProps({ handleScrollToSelection: () => true });
  cleanups.push(() => app.destroy());
  const enter = () => app.view.dom.dispatchEvent(new KeyboardEvent("keydown",
    { key: "Enter", code: "Enter", keyCode: 13, bubbles: true, cancelable: true }));
  const type = (text: string) => app.view.dispatch(app.view.state.tr.insertText(text));
  const settle = async () => { await vi.waitFor(() => expect(root.textContent).not.toContain("projection en attente")); };
  return { app, root, invoke, pending, enter, type, settle, reject: () => { reject = true; }, snapshot: () => s };
}

it("après migration l'édition manuelle et son annulation restent disponibles", async () => {
  const e = await editor({ ...empty, preferencesMigrated:true, source:"Note", blocks:[
    {from:0,to:4,raw:"Note",kind:"paragraph"},
  ] });
  e.app.view.dispatch(e.app.view.state.tr.setSelection(TextSelection.atEnd(e.app.view.state.doc)));
  e.type(" modifiée");
  await e.settle();
  expect(e.snapshot().source).toBe("Note modifiée");
  e.root.querySelector<HTMLButtonElement>('[data-action="undo"]')!.click();
  await vi.waitFor(() => expect(e.snapshot().source).toBe("Note"));
});

it("Entrée réelle PM en fin de note garde deux paragraphes, curseur et focus après ack", async () => {
  const e = await editor("un");
  e.app.view.dispatch(e.app.view.state.tr.setSelection(TextSelection.atEnd(e.app.view.state.doc)));
  e.app.view.focus();
  expect(e.enter()).toBe(false); // handled by real PM baseKeymap
  await e.settle();
  expect(e.snapshot().source).toBe("un\n\n");
  expect(e.app.view.state.doc.childCount).toBe(2);
  expect(e.app.view.state.selection.$from.parent.textContent).toBe("");
  expect(e.app.view.hasFocus()).toBe(true);
  e.type("deux");
  await e.settle();
  expect(e.snapshot().source).toBe("un\n\ndeux");
  expect(e.app.view.state.doc.child(1).textContent).toBe("deux");
  expect(e.app.view.hasFocus()).toBe(true);
});

it("saisie rapide → Entrée → saisie avant ack, préconditions, undo/redo acquittés sans perte", async () => {
  const e = await editor("", true);
  e.app.view.focus();
  e.type("un"); e.enter(); e.type("deux");
  expect(e.app.view.state.doc.childCount).toBe(2);
  expect(e.app.view.state.doc.child(1).textContent).toBe("deux");
  for (let i = 0; i < 3; i++) {
    await vi.waitFor(() => expect(e.pending.length).toBeGreaterThan(i));
    e.pending[i]!();
  }
  await e.settle();
  expect(e.snapshot().source).toBe("un\n\ndeux");
  expect(e.app.view.hasFocus()).toBe(true);
  const history = async (action: string, index: number) => {
    e.root.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)!.click();
    await vi.waitFor(() => expect(e.pending.length).toBe(index + 1));
    e.pending[index]!();
    await vi.waitFor(() => expect(e.root.querySelector<HTMLButtonElement>('[data-action="undo"]')!.disabled).toBe(false));
  };
  await history("undo", 3);
  expect(e.snapshot().source).toBe("un\n\n");
  expect(e.app.view.state.doc.childCount).toBe(2);
  await history("undo", 4);
  expect(e.snapshot().source).toBe("un");
  expect(e.app.view.state.doc.childCount).toBe(1);
  await history("redo", 5);
  expect(e.snapshot().source).toBe("un\n\n");
  expect(e.app.view.state.doc.childCount).toBe(2);
});

it("refus natif récupère toute la projection rapide sans envoyer la suite ni perdre le focus", async () => {
  const e = await editor("un", true);
  e.app.view.dispatch(e.app.view.state.tr.setSelection(TextSelection.atEnd(e.app.view.state.doc)));
  e.app.view.focus(); e.enter(); e.type("deux");
  e.reject(); e.pending[0]!();
  await vi.waitFor(() => expect(e.root.textContent).toContain("aucun rejeu"));
  expect(e.root.querySelector('pre[aria-label="Saisie non acceptée récupérable en mémoire"]')!.textContent).toBe("un\n\ndeux");
  expect(e.snapshot().source).toBe("un");
  expect(e.app.view.hasFocus()).toBe(true);
  expect(e.invoke.mock.calls.filter(c => c[0] === "prototype_manual")).toHaveLength(1);
});

it("Entrée au milieu d'un paragraphe UTF-8 garde la sélection après ack", async () => {
  const e = await editor("été fin");
  e.app.view.dispatch(e.app.view.state.tr.setSelection(TextSelection.create(e.app.view.state.doc, 4)));
  e.app.view.focus(); e.enter();
  await e.settle();
  expect(e.snapshot().source).toBe("été\n\n fin");
  expect(e.app.view.state.selection.$from.parentOffset).toBe(0);
  e.type("deux");
  await e.settle();
  expect(e.snapshot().source).toBe("été\n\ndeux fin");
  expect(e.app.view.hasFocus()).toBe(true);
});

it("les paragraphes vides et le bloc opaque hors cible survivent aux ack et au polling", async () => {
  vi.useFakeTimers();
  const source = "un\r\n\r\n```rs\r\nopaque()\r\n```\r\n";
  const raw = "```rs\r\nopaque()\r\n```";
  const e = await editor({ ...empty, source, blocks: [
    { from: 0, to: 2, raw: "un", kind: "paragraph" },
    { from: 6, to: 6 + new TextEncoder().encode(raw).length, raw, kind: "opaque" },
  ] });
  e.app.view.dispatch(e.app.view.state.tr.setSelection(TextSelection.create(e.app.view.state.doc, 3)));
  e.app.view.focus(); e.enter(); e.enter(); e.type("deux");
  await vi.advanceTimersByTimeAsync(0);
  expect(e.snapshot().source).toBe("un\n\n\n\ndeux\r\n\r\n```rs\r\nopaque()\r\n```\r\n");
  const last = e.app.view.state.doc.lastChild!;
  expect(last.type.name).toBe("opaque");
  expect(last.attrs.raw).toBe(raw);
  await vi.advanceTimersByTimeAsync(2000);
  expect(e.app.view.state.doc.childCount).toBe(4);
  expect(e.app.view.hasFocus()).toBe(true);
});

it("Fichier : vrais clics disclosure, nouvelle note explicite, refus save sans chemin persistant et succès", async () => {
  vi.useFakeTimers();
  const e = await editor();
  const files = e.root.querySelector<HTMLButtonElement>("nav button[aria-expanded]")!;
  const action = async (id: string) => {
    files.click();
    const b = e.root.querySelector<HTMLButtonElement>(`[data-action="${id}"]`)!;
    b.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, cancelable: true }));
    b.click();
    await vi.advanceTimersByTimeAsync(0);
    expect(files.getAttribute("aria-expanded")).toBe("false");
  };
  await action("new");
  expect(e.snapshot().noteId).toBe(2);
  expect(e.root.textContent).toContain("Nouvelle note créée");
  await action("save");
  expect(e.root.textContent).toContain("Chemin absolu vers un fichier .md requis");
  await vi.advanceTimersByTimeAsync(2000);
  expect(e.root.textContent).toContain("Chemin absolu vers un fichier .md requis");
  await action("save-current");
  expect(e.root.textContent).toContain("Choisir une destination");
  const path = e.root.querySelector<HTMLInputElement>('input[aria-label="Destination Markdown locale"]')!;
  path.value = "C:\\synthetic\\note.md";
  await action("save");
  expect(e.root.textContent).toContain("Nouvelle copie Markdown enregistrée");
  e.type("ne pas perdre");
  await vi.advanceTimersByTimeAsync(0);
  await action("new");
  expect(e.snapshot().source).toBe("ne pas perdre");
  expect(e.root.textContent).toContain("Sauvegarder la note modifiée");
});
