import { Schema } from "prosemirror-model";
import { EditorView, type DirectEditorProps } from "prosemirror-view";
import { defaultMarkdownParser } from "prosemirror-markdown";
import type { Snapshot } from "./contracts";

// Source URLs are data only: neither links nor images create active DOM resources.
export const schema = new Schema({
  nodes: defaultMarkdownParser.schema.spec.nodes
    .update("image", { ...defaultMarkdownParser.schema.spec.nodes.get("image")!,
      toDOM: node => ["span", { "data-note-image": "true" }, `${node.attrs.alt || "Image"} (image non chargée)`],
    }).addBefore("paragraph", "opaque", {
      group: "block", atom: true, selectable: true, attrs: { raw: { default: "" } },
      toDOM: node => ["pre", { "data-opaque": "true", contenteditable: "false" }, node.attrs.raw as string],
    }),
  marks: defaultMarkdownParser.schema.spec.marks.update("link", {
    ...defaultMarkdownParser.schema.spec.marks.get("link")!,
    toDOM: () => ["span", { "data-note-link": "true" }, 0],
  }),
});

export function documentFor(s: Snapshot) {
  return schema.nodes.doc!.create(null, s.blocks.map(block => {
    if (block.kind === "opaque") return schema.nodes.opaque!.create({ raw: block.raw });
    const parsed = defaultMarkdownParser.parse(block.raw).firstChild;
    return parsed ? schema.nodeFromJSON(parsed.toJSON()) : schema.nodes.paragraph!.create();
  }));
}

export function createNoteView(root: HTMLElement, props: DirectEditorProps) {
  return new EditorView(root, { ...props, handleDOMEvents: {
    // Consume paste before ProseMirror parses HTML into a detached DOM tree.
    paste(view, event) {
      event.preventDefault();
      if (view.editable) {
        const text = event.clipboardData?.getData("text/plain");
        if (text) view.dispatch(view.state.tr.insertText(text));
      }
      return true;
    },
    drop(_view, event) { event.preventDefault(); return true; },
  } });
}
