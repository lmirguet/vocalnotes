// @vitest-environment jsdom
import { describe, it, expect } from "vitest";
import { EditorState } from "prosemirror-state";
import { defaultMarkdownSerializer } from "prosemirror-markdown";
import { createNoteView, documentFor, schema } from "./NoteView";
import type { Snapshot } from "./DesktopShell";

function snapshot(raw: string, kind = "paragraph"): Snapshot {
  return { noteId: 1, manualPeriod: 1, revision: 0, generation: 1, mode: "manual",
    dictationMode: "content", context: "", instructions: "", dictationTransitionPending: false, inputGeneration: 1,
    source: raw, dirty: false, message: "", blocks: [{ from: 0, to: new TextEncoder().encode(raw).length, raw, kind }] };
}

describe("rendu Markdown sans contenu actif", () => {
  it("préserve les titres, le gras et l'italique au chargement et à la sérialisation", () => {
    const raw = "# Titre\n\n**gras** et *italique*";
    const state = snapshot(raw);
    state.blocks = [
      { from: 0, to: 7, raw: "# Titre", kind: "heading" },
      { from: 9, to: raw.length, raw: "**gras** et *italique*", kind: "paragraph" },
    ];
    const doc = documentFor(state);
    expect(doc.child(0)!.type.name).toBe("heading");
    expect(doc.child(1)!.firstChild!.marks.map(mark => mark.type.name)).toEqual(["strong"]);
    expect(doc.child(1)!.lastChild!.marks.map(mark => mark.type.name)).toEqual(["em"]);
    expect(defaultMarkdownSerializer.serialize(doc)).toBe(raw);
  });

  it.each([
    '[lien](javascript:alert%281%29)', '[lien](https://example.test/)',
    '![image](https://example.test/tracker.png)', '![image](file:///tmp/private.png)',
    '<img src="https://example.test/x" onerror="alert(1)"><script>alert(1)</script>',
  ])("ne crée aucune ressource ni navigation depuis %s", raw => {
    const root = document.createElement("section");
    const view = createNoteView(root, { state: EditorState.create({ schema, doc: documentFor(snapshot(raw)) }) });
    try {
      expect(root.querySelector("img[src],script,iframe,object,a[href],[src],[onerror]")).toBeNull();
      expect(root.textContent!.length).toBeGreaterThan(0);
    } finally { view.destroy(); }
  });
  it("affiche un îlot opaque littéralement et sans sérialisation", () => {
    const raw = '<img src="https://example.test/x">\r\n```html\r\n<script>x</script>\r\n```';
    const root = document.createElement("section");
    const view = createNoteView(root, { state: EditorState.create({ schema, doc: documentFor(snapshot(raw, "opaque")) }) });
    try {
      expect(root.querySelector('[data-opaque]')?.textContent).toBe(raw);
      expect(root.querySelector("img,script")).toBeNull();
    } finally { view.destroy(); }
  });
  it("ignore le HTML du presse-papiers et insère seulement le texte brut", () => {
    const root = document.createElement("section");
    const view = createNoteView(root, { state: EditorState.create({ schema }) });
    try {
      const event = new Event("paste", { bubbles: true, cancelable: true });
      const getData = (type: string) => type === "text/plain" ? "Texte sûr" : '<img src="https://example.test/x">';
      Object.defineProperty(event, "clipboardData", { value: { getData } });
      view.dom.dispatchEvent(event);
      expect(view.state.doc.textContent).toBe("Texte sûr");
      expect(root.querySelector("img")).toBeNull();
    } finally { view.destroy(); }
  });
});
