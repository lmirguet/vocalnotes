import { Schema, Fragment, type Node as PMNode } from "prosemirror-model";
import { EditorState, TextSelection, type Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { defaultMarkdownParser, defaultMarkdownSerializer } from "prosemirror-markdown";
import { toggleMark, setBlockType, baseKeymap } from "prosemirror-commands";
import { wrapInList } from "prosemirror-schema-list";
import { keymap } from "prosemirror-keymap";

type Block = { from: number; to: number; raw: string; kind: string };
export type Snapshot = {
  noteId: number; manualPeriod: number; revision: number; generation: number;
  mode: string; source: string; dirty: boolean; message: string; blocks: Block[];
  sttRequests?: number; llmRequests?: number; priorityRequests?: number; audioMs?: number;
  sttModel?: string; llmModel?: string;
  lastSttFinalMs?: number | null; lastLlmMs?: number | null;
  lastPriorityMs?: number | null; lastCommitUs?: number | null;
  closePending?: boolean;
};
type Patch = { from: number; to: number; expected: string; text: string };
type Proposal = {
  transactionId: string; noteId: number; manualPeriod: number;
  baseRevision: number; patches: Patch[];
};
type Ack = { transactionId: string; accepted: boolean; message: string; snapshot: Snapshot };
type Invoke = (command: string, args?: Record<string, unknown>) => Promise<unknown>;
type History = { forward: Patch[]; backward: Patch[];
  beforeState: EditorState; afterState: EditorState; beforeBlocks: Block[]; afterBlocks: Block[] };
type Pending = { proposal: Proposal; after: Snapshot; history: History; structural: boolean };

const schema = new Schema({
  nodes: defaultMarkdownParser.schema.spec.nodes.addBefore("paragraph", "opaque", {
    group: "block", atom: true, selectable: true, attrs: { raw: { default: "" } },
    toDOM: node => ["pre", { "data-opaque": "true", contenteditable: "false" }, node.attrs.raw as string],
  }),
  marks: defaultMarkdownParser.schema.spec.marks,
});
const encoder = new TextEncoder();
const bytes = (s: string) => encoder.encode(s).length;
// fatal/TextDecoder variants vary across test/desktop webviews; all native source is UTF-8.
const slice = (s: string, from: number, to: number) => new TextDecoder().decode(encoder.encode(s).slice(from, to));

function documentFor(s: Snapshot): PMNode {
  const nodes = s.blocks.map(block => {
    if (block.kind === "opaque") return schema.nodes.opaque!.create({ raw: block.raw });
    const parsed = defaultMarkdownParser.parse(block.raw).firstChild;
    return parsed ? schema.nodeFromJSON(parsed.toJSON()) : schema.nodes.paragraph!.create();
  });
  return schema.nodes.doc!.create(null, nodes);
}

function inverse(patches: Patch[]): Patch[] {
  let shift = 0;
  return [...patches].sort((a, b) => a.from - b.from).map(p => {
    const result = { from: p.from + shift, to: p.from + shift + bytes(p.text), expected: p.text, text: p.expected };
    shift += bytes(p.text) - (p.to - p.from);
    return result;
  });
}

function predicted(s: Snapshot, patches: Patch[]): Snapshot {
  let source = s.source;
  for (const p of [...patches].sort((a, b) => b.from - a.from)) {
    source = slice(source, 0, p.from) + p.text + slice(source, p.to, bytes(source));
  }
  const blocks = s.blocks.map(b => {
    const replacement = patches.find(p => p.from === b.from && p.to === b.to);
    const shift = patches.filter(p => p.to <= b.from && p !== replacement)
      .reduce((n, p) => n + bytes(p.text) - (p.to - p.from), 0);
    const raw = replacement?.text ?? b.raw;
    const kind = replacement ? (raw.startsWith("#") ? "heading" : "paragraph") : b.kind;
    return { from: b.from + shift, to: b.from + shift + bytes(raw), raw, kind };
  });
  return { ...s, source, blocks, revision: s.revision + 1, dirty: true };
}

export async function mountPrototype(root: HTMLElement, rawInvoke: Invoke) {
  const invoke = <T = unknown>(command: string, args?: Record<string, unknown>) => rawInvoke(command, args) as Promise<T>;
  let canonical = await invoke<Snapshot>("prototype_snapshot");
  let projected = canonical;
  let disposed = false;
  let recovery = "";
  let serial = 0;
  let sending = false;
  let synchronized = true;
  let acting = false;
  let feedback = canonical.message;
  const queue: Pending[] = [];
  const undo: History[] = [];
  const redo: History[] = [];
  const nonce = crypto.randomUUID();

  const heading = document.createElement("h1");
  heading.textContent = "Bloc-notes vocal — prototype T006 non recetté";
  const menu = document.createElement("nav");
  menu.setAttribute("aria-label", "Menus");
  menu.className = "menu-bar";
  const groups: Array<{ trigger: HTMLButtonElement; panel: HTMLElement }> = [];
  let opened: typeof groups[number] | undefined;
  // Disclosure buttons, not ARIA menu/menuitem: panels contain an editable path
  // and help prose. Enter/Space and Tab keep their ordinary native semantics.
  function closeMenu(returnFocus = false) {
    if (!opened) return;
    const previous = opened;
    previous.panel.hidden = true;
    previous.trigger.setAttribute("aria-expanded", "false");
    opened = undefined;
    if (returnFocus) previous.trigger.focus();
  }
  function menuClick(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const group = groups.find(g => g.trigger === target.closest("button"));
    if (group) {
      const wasOpen = opened === group;
      closeMenu();
      group.trigger.focus();
      if (!wasOpen) {
        group.panel.hidden = false;
        group.trigger.setAttribute("aria-expanded", "true");
        opened = group;
      }
    } else if (opened?.panel.contains(target) && target.closest("button[data-action]:not(:disabled)")) {
      closeMenu(true);
    }
  }
  function menuKey(event: KeyboardEvent) {
    const index = groups.findIndex(g => g.trigger === event.target);
    if (index < 0) return; // arrows in the path input edit text, never move menus
    const next = event.key === "ArrowRight" ? (index + 1) % groups.length
      : event.key === "ArrowLeft" ? (index + groups.length - 1) % groups.length
        : event.key === "Home" ? 0 : event.key === "End" ? groups.length - 1 : -1;
    if (next < 0) return;
    event.preventDefault();
    closeMenu();
    groups[next]!.trigger.focus();
  }
  function outsideClick(event: MouseEvent) {
    if (event.target instanceof Node && !menu.contains(event.target)) closeMenu();
  }
  function escapeMenu(event: KeyboardEvent) {
    if (event.key === "Escape" && opened && event.target instanceof Node && menu.contains(event.target)) {
      event.preventDefault();
      closeMenu(true);
    }
  }
  function leaveMenu(event: FocusEvent) {
    if (!(event.relatedTarget instanceof Node) || !menu.contains(event.relatedTarget)) closeMenu();
  }
  menu.addEventListener("click", menuClick);
  menu.addEventListener("keydown", menuKey);
  menu.addEventListener("focusout", leaveMenu);
  document.addEventListener("click", outsideClick);
  document.addEventListener("keydown", escapeMenu);
  const toolbar = document.createElement("div");
  toolbar.setAttribute("role", "toolbar");
  toolbar.setAttribute("aria-label", "Mise en forme et vocal");
  const status = document.createElement("p");
  status.setAttribute("role", "status");
  const notice = document.createElement("p");
  notice.textContent = "Prototype non certifié GNOME/Windows. Sauvegarde en nouvelle copie exclusive uniquement ; " +
    "les blocs non pris en charge restent opaques. Le vocal exige la surveillance OS native disponible. " +
    "Audio transmis à Voxtral, passages finalisés/intention/sections éditables à Mistral Large ; " +
    "les provisoires servent aussi à l'interruption naturelle (requêtes supplémentaires). " +
    "Tarifs et conservation fournisseur inconnus : aucun contenu sensible. " +
    "Attention aux paroles de tiers en arrière-plan. Aucune notification hors fenêtre.";
  const editor = document.createElement("section");
  editor.setAttribute("aria-label", "Note");
  const recovered = document.createElement("pre");
  recovered.hidden = true;
  recovered.setAttribute("aria-label", "Saisie non acceptée récupérable en mémoire");

  function show(message = feedback) {
    feedback = message;
    status.textContent = `${projected.mode === "manual" ? "Manuel" : projected.mode === "paused" ? "Pause" : "Vocal"} — ${message}` +
      ` — tentatives STT : ${canonical.sttRequests ?? 0}, LLM : ${canonical.llmRequests ?? 0}, priorité : ${canonical.priorityRequests ?? 0}` +
      ` ; audio transmis : ${canonical.audioMs ?? 0} ms ; coût inconnu.` +
      (canonical.sttModel ? ` STT servi : ${canonical.sttModel}.` : "") +
      (canonical.llmModel ? ` LLM servi : ${canonical.llmModel}.` : "") +
      ` Derniers délais : flush→done ${canonical.lastSttFinalMs ?? "non mesuré"} ms ; ` +
      `LLM ${canonical.lastLlmMs ?? "non mesuré"} ms ; priorité ${canonical.lastPriorityMs ?? "non mesuré"} ms ; ` +
      `commit ${canonical.lastCommitUs ?? "non mesuré"} µs.` +
      (queue.length ? " — projection en attente, non sauvegardée." : canonical.dirty ? " — note modifiée, non sauvegardée." : "");
    undoButton.disabled = projected.mode !== "manual" || queue.length > 0 || !undo.length;
    redoButton.disabled = projected.mode !== "manual" || queue.length > 0 || !redo.length;
    startButton.disabled = !consent.checked || canonical.closePending === true || projected.mode === "voice" || sending || queue.length > 0;
    resumeButton.disabled = !consent.checked || projected.mode !== "paused";
  }
  function restore(s: Snapshot, message: string) {
    const focused = view.hasFocus();
    const { anchor, head } = view.state.selection;
    synchronized = true;
    canonical = projected = s;
    queue.length = 0;
    undo.length = redo.length = 0;
    let state = editorState(s);
    state = state.apply(state.tr.setSelection(TextSelection.near(
      state.doc.resolve(Math.min(head, state.doc.content.size)))));
    if (anchor !== head && anchor <= state.doc.content.size && head <= state.doc.content.size) {
      state = state.apply(state.tr.setSelection(TextSelection.create(state.doc, anchor, head)));
    }
    view.updateState(state);
    view.setProps({ editable: () => synchronized && projected.mode === "manual" });
    if (focused && projected.mode === "manual") view.focus();
    recovered.textContent = recovery;
    recovered.hidden = !recovery;
    show(message);
  }

  async function drain() {
    if (sending || disposed) return;
    sending = true;
    try {
      while (queue.length && !disposed) {
        const pending = queue[0]!;
        let ack: Ack;
        try {
          ack = await invoke<Ack>("prototype_manual", { proposal: pending.proposal });
        } catch {
          recovery = projected.source;
          const current = await invoke<Snapshot>("prototype_snapshot");
          restore(current, "Échec de transaction ; saisie récupérable, historique réinitialisé, aucun rejeu.");
          break;
        }
        if (disposed) break;
        if (!ack.accepted || ack.transactionId !== pending.proposal.transactionId
          || ack.snapshot.noteId !== pending.proposal.noteId
          || ack.snapshot.manualPeriod !== pending.proposal.manualPeriod
          || ack.snapshot.revision !== pending.after.revision
          || ack.snapshot.source !== pending.after.source) {
          recovery = projected.source;
          const current = await invoke<Snapshot>("prototype_snapshot");
          restore(current, "Refus ou divergence ; saisie récupérable, historique réinitialisé, aucun rejeu.");
          break;
        }
        canonical = ack.snapshot;
        queue.shift();
        undo.push(pending.history);
        redo.length = 0;
        // Keep the PM document/selection and blank insertion paragraphs. Markdown
        // parsing omits these, but the source/revision must match the native ack.
        if (!queue.length) projected = { ...canonical, blocks: projected.blocks };
        show(ack.message);
      }
    } catch {
      // Do not continue sending on an unknown canonical state.
      queue.length = 0;
      recovery = projected.source;
      recovered.textContent = recovery;
      recovered.hidden = false;
      synchronized = false;
      view.setProps({ editable: () => false });
      show("Échec du cœur : édition suspendue ; saisie récupérable en mémoire, aucun rejeu.");
    } finally { sending = false; show(); }
  }

  function enqueue(patches: Patch[], structural: boolean, beforeState: EditorState, blocks?: Block[]) {
    const proposal: Proposal = {
      transactionId: `${nonce}:${++serial}`, noteId: projected.noteId,
      manualPeriod: projected.manualPeriod, baseRevision: projected.revision, patches,
    };
    const beforeBlocks = projected.blocks;
    projected = predicted(projected, patches);
    if (blocks) projected = { ...projected, blocks };
    queue.push({ proposal, after: projected, history: {
      forward: patches, backward: inverse(patches), beforeState, afterState: view.state,
      beforeBlocks, afterBlocks: projected.blocks,
    }, structural });
    show("Modification proposée au cœur.");
    void drain();
  }

  function dispatch(tr: Transaction) {
    if (!tr.docChanged) { view.updateState(view.state.apply(tr)); return; }
    if (acting) { show("Action en cours ; attendre son résultat avant de modifier la note."); return; }
    if (!synchronized || projected.mode !== "manual") return;
    const beforeState = view.state;
    if (sending && !queue.length) {
      show("Attendre l'acquittement de l'annulation avant de modifier la note.");
      return;
    }
    if (queue.length >= 32) {
      recovery = tr.doc.textContent;
      recovered.textContent = recovery;
      recovered.hidden = false;
      show("File pleine : saisie récupérable, aucune perte silencieuse.");
      return;
    }
    if (tr.doc.childCount !== view.state.doc.childCount) {
      let first = 0;
      while (first < Math.min(tr.doc.childCount, view.state.doc.childCount)
        && tr.doc.child(first).eq(view.state.doc.child(first))) first++;
      let oldEnd = view.state.doc.childCount;
      let newEnd = tr.doc.childCount;
      while (oldEnd > first && newEnd > first && view.state.doc.child(oldEnd - 1).eq(tr.doc.child(newEnd - 1))) {
        oldEnd--; newEnd--;
      }
      const oldBlocks = projected.blocks.slice(first, oldEnd);
      if (oldBlocks.some(b => b.kind === "opaque")) { show("Bloc opaque : modification de structure refusée."); return; }
      const nodes: PMNode[] = [];
      for (let i = first; i < newEnd; i++) {
        if (tr.doc.child(i).type.name === "opaque") { show("Bloc opaque : modification refusée."); return; }
        nodes.push(defaultMarkdownParser.schema.nodeFromJSON(tr.doc.child(i).toJSON()));
      }
      // Serialize only the changed span, individually: serializer(doc) drops an
      // empty final paragraph. Two newlines explicitly encode the insertion point.
      const raws = nodes.map(node => defaultMarkdownSerializer.serialize(
        defaultMarkdownParser.schema.node("doc", null, [node])));
      let text = raws.join("\n\n");
      const from = oldBlocks[0]?.from ?? (first ? projected.blocks[first - 1]!.to : 0);
      const to = oldBlocks.at(-1)?.to ?? from;
      if (!oldBlocks.length && nodes.length) text = first ? `\n\n${text}` : `${text}\n\n`;
      const shift = bytes(text) - (to - from);
      let offset = from + (!oldBlocks.length && first && nodes.length ? 2 : 0);
      const replacements = raws.map((raw, i) => {
        const node = nodes[i]!;
        const block = { from: offset, to: offset + bytes(raw), raw,
          kind: node.type.name === "paragraph" || node.type.name === "heading" ? node.type.name : "opaque" };
        offset += bytes(raw) + 2;
        return block;
      });
      const blocks = [...projected.blocks.slice(0, first), ...replacements,
        ...projected.blocks.slice(oldEnd).map(b => ({ ...b, from: b.from + shift, to: b.to + shift }))];
      view.updateState(view.state.apply(tr));
      enqueue([{ from, to, expected: slice(projected.source, from, to), text }], true, beforeState, blocks);
      return;
    }
    const patches: Patch[] = [];
    let structural = false;
    for (let i = 0; i < tr.doc.childCount; i++) {
      const old = view.state.doc.child(i);
      const next = tr.doc.child(i);
      if (old.eq(next)) continue;
      const block = projected.blocks[i];
      if (!block || block.kind === "opaque" || next.type.name === "opaque") {
        show("Bloc opaque : modification refusée avant altération.");
        return;
      }
      const doc = defaultMarkdownParser.schema.node("doc", null,
        Fragment.from(defaultMarkdownParser.schema.nodeFromJSON(next.toJSON())));
      const text = defaultMarkdownSerializer.serialize(doc);
      structural ||= old.type !== next.type;
      patches.push({ from: block.from, to: block.to, expected: block.raw, text });
    }
    view.updateState(view.state.apply(tr));
    if (patches.length) {
      const blocks = predicted(projected, patches).blocks.map((b, i) => ({
        ...b, kind: tr.doc.child(i).type.name === "paragraph" || tr.doc.child(i).type.name === "heading"
          ? tr.doc.child(i).type.name : "opaque",
      }));
      enqueue(patches, structural, beforeState, blocks);
    }
  }

  function editorState(s: Snapshot) {
    return EditorState.create({ schema, doc: documentFor(s), plugins: [
      keymap({
        "Mod-z": () => { if (projected.mode !== "manual") return false; void history("undo"); return true; },
        "Mod-y": () => { if (projected.mode !== "manual") return false; void history("redo"); return true; },
        "Shift-Mod-z": () => { if (projected.mode !== "manual") return false; void history("redo"); return true; },
      }), keymap(baseKeymap),
    ] });
  }
  const view = new EditorView(editor, {
    state: editorState(projected),
    editable: () => synchronized && projected.mode === "manual",
    dispatchTransaction: dispatch,
    attributes: { "aria-label": "Note visuelle", role: "textbox", "aria-multiline": "true" },
  });

  function button(parent: HTMLElement, label: string, action: string, run: () => void) {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = label;
    b.dataset.action = action;
    b.addEventListener("click", run);
    b.addEventListener("mousedown", event => event.preventDefault());
    parent.append(b);
    return b;
  }
  function group(label: string) {
    const container = document.createElement("div");
    container.className = "menu-group";
    const trigger = document.createElement("button");
    trigger.type = "button";
    trigger.className = "menu-trigger";
    trigger.textContent = label;
    const panel = document.createElement("section");
    panel.className = "menu-panel";
    panel.id = `menu-${nonce}-${groups.length}`;
    panel.setAttribute("aria-label", label);
    panel.hidden = true;
    trigger.setAttribute("aria-controls", panel.id);
    trigger.setAttribute("aria-expanded", "false");
    groups.push({ trigger, panel });
    container.append(trigger, panel);
    menu.append(container);
    return panel;
  }
  async function act(action: Record<string, string | boolean>) {
    if (acting || sending || queue.length) { show("Attendre l'acquittement des modifications avant cette action."); return; }
    acting = true;
    try {
      const s = await invoke<Snapshot>("prototype_action", { action });
      const discarded = recovery !== "" && (s.noteId !== canonical.noteId || s.manualPeriod !== canonical.manualPeriod);
      const changed = s.noteId !== canonical.noteId || s.manualPeriod !== canonical.manualPeriod || s.revision !== canonical.revision;
      if (changed) {
        recovery = "";
        restore(s, discarded ? "Changement effectué ; ancienne saisie non acceptée supprimée de la mémoire de l'interface." : s.message);
      } else {
        canonical = s;
        projected = { ...s, blocks: projected.blocks };
        show(s.message); // A save does not erase confirmed manual undo or recovery.
      }
    } catch (error) {
      // A failed open still invalidates the old generation: ALWAYS resynchronise.
      // Only exact static native diagnostics are displayable; never expose payloads,
      // arbitrary exception strings, paths, note text or provider responses.
      const reasons = [
        "Sauvegarder la note modifiée avant de changer de note.",
        "Chemin absolu vers un fichier .md requis.",
        "Choisir une destination Markdown locale avant la sauvegarde ; aucun effet.",
        "Création exclusive impossible : choisir un nouveau fichier Markdown local.",
        "Écriture incomplète : note conservée en mémoire, vérifier la copie.",
        "Note inaccessible.", "Lecture UTF-8 impossible.",
        "Note ordinaire UTF-8 de moins de 1 Mio requise.",
      ];
      const message = typeof error === "string" && reasons.includes(error) ? error
        : "Échec de l'action : note conservée. Vérifier destination et sauvegarde.";
      try {
        const s = await invoke<Snapshot>("prototype_snapshot");
        if (s.noteId === canonical.noteId && s.manualPeriod === canonical.manualPeriod
          && s.revision === canonical.revision && s.source === canonical.source) {
          canonical = s; projected = { ...s, blocks: projected.blocks }; show(message);
        } else restore(s, message);
      }
      catch { show("Échec du cœur ; aucune réussite annoncée."); }
    } finally { acting = false; }
  }
  const files = group("Fichier");
  button(files, "Nouvelle note", "new", () => void act({ type: "new" }));
  const path = document.createElement("input");
  path.placeholder = "Chemin absolu vers un fichier .md local";
  path.setAttribute("aria-label", "Destination Markdown locale");
  files.append(path);
  button(files, "Ouvrir", "open", () => void act({ type: "open", path: path.value }));
  button(files, "Enregistrer une nouvelle copie", "save", () => void act({ type: "save_copy", path: path.value }));
  button(files, "Enregistrer une copie à la destination courante", "save-current", () => void act({ type: "save" }));
  button(files, "Fermer après sauvegarde", "close", () => {
    if (sending || queue.length) { show("Attendre les acquittements avant de fermer."); return; }
    if (recovery && !window.confirm("La saisie non acceptée n'est pas dans la note. Copiez-la avant de fermer. Fermer quand même ?")) return;
    void invoke<Snapshot>("prototype_finish_close").then(s => restore(s, s.message))
      .catch(() => show("Fermeture impossible ; note conservée. Choisir une destination ou vérifier la sauvegarde."));
  });
  button(files, "Annuler la fermeture", "cancel-close", () => void act({ type: "cancel_close" }));
  button(files, "Copier la source acceptée", "copy", () => {
    void navigator.clipboard.writeText(canonical.source).catch(() => show("Échec de copie ; note inchangée."));
  });
  const edits = group("Édition");
  async function history(direction: "undo" | "redo") {
    if (acting || sending || queue.length || projected.mode !== "manual") return;
    const stack = direction === "undo" ? undo : redo;
    const entry = stack.at(-1);
    if (!entry) return;
    const patches = direction === "undo" ? entry.backward : entry.forward;
    const proposal: Proposal = {
      transactionId: `${nonce}:${++serial}`, noteId: canonical.noteId,
      manualPeriod: canonical.manualPeriod, baseRevision: canonical.revision, patches,
    };
    sending = true;
    const focused = view.hasFocus();
    try {
      const ack = await invoke<Ack>("prototype_manual", { proposal });
      if (!ack.accepted || ack.transactionId !== proposal.transactionId
        || ack.snapshot.noteId !== proposal.noteId || ack.snapshot.manualPeriod !== proposal.manualPeriod
        || ack.snapshot.revision !== proposal.baseRevision + 1
        || ack.snapshot.source !== predicted(canonical, patches).source) {
        recovery = projected.source;
        restore(await invoke<Snapshot>("prototype_snapshot"), "Annulation refusée ; historique réinitialisé sans rejeu.");
      } else {
        stack.pop();
        (direction === "undo" ? redo : undo).push(entry);
        canonical = ack.snapshot;
        projected = { ...canonical, blocks: direction === "undo" ? entry.beforeBlocks : entry.afterBlocks };
        view.updateState(direction === "undo" ? entry.beforeState : entry.afterState);
        if (focused) view.focus();
        show(ack.message);
      }
    } catch {
      recovery = projected.source;
      try { restore(await invoke<Snapshot>("prototype_snapshot"), "Échec d'annulation ; historique réinitialisé sans rejeu."); }
      catch {
        recovered.textContent = recovery;
        recovered.hidden = false;
        synchronized = false;
        show("Échec du cœur ; saisie récupérable en mémoire, recharger avant édition.");
        return;
      }
    } finally {
      sending = false;
      view.setProps({ editable: () => synchronized && projected.mode === "manual" });
    }
  }
  const undoButton = button(edits, "Annuler", "undo", () => void history("undo"));
  const redoButton = button(edits, "Rétablir", "redo", () => void history("redo"));
  const help = group("Aide");
  const helpText = document.createElement("p");
  helpText.textContent = "Audio, transcription et intention restent temporaires. " +
    "Micro ouvert seulement après accord, démarrage explicite et surveillance OS établie. Une note seule est enregistrée. " +
    "Fermeture modifiée avec destination : nouvelle copie vocalnotes-fermeture-<identifiant>.md dans le même dossier ; " +
    "sans destination ou en cas d'erreur, fermeture suspendue et note accessible. " +
    "Coût et conservation Mistral inconnus ; pas de données sensibles. Aucun indicateur hors fenêtre.";
  help.append(helpText);
  for (const [label, command] of [
    ["Gras", toggleMark(schema.marks.strong!)], ["Italique", toggleMark(schema.marks.em!)],
    ["Titre", setBlockType(schema.nodes.heading!, { level: 1 })],
    ["Paragraphe", setBlockType(schema.nodes.paragraph!)],
  ] as const) {
    button(toolbar, label, label, () => {
      if (projected.mode === "manual") command(view.state, view.dispatch, view);
    });
  }
  button(toolbar, "Liste", "list", () => {
    if (projected.mode === "manual") wrapInList(schema.nodes.bullet_list!)(view.state, view.dispatch, view);
  });
  const consentLabel = document.createElement("label");
  const consent = document.createElement("input");
  consent.type = "checkbox";
  consent.dataset.consent = "voice";
  consentLabel.append(consent, document.createTextNode("J'autorise le micro et les envois Mistral décrits ci-dessus, coût inconnu."));
  toolbar.append(consentLabel);
  consent.addEventListener("change", () => show());
  const startButton = button(toolbar, "Démarrer le vocal", "start", () => void act({ type: "start", consent: consent.checked, resume: false }));
  const resumeButton = button(toolbar, "Reprendre explicitement", "resume", () => void act({ type: "start", consent: consent.checked, resume: true }));
  button(toolbar, "Pause", "pause", () => void act({ type: "pause" }));
  button(toolbar, "Arrêter le vocal", "stop", () => void act({ type: "stop" }));
  const keyForm = document.createElement("section");
  keyForm.setAttribute("aria-label", "Clé Mistral — trousseau du système");
  const keyLabel = document.createElement("label");
  keyLabel.textContent = "Clé Mistral (trousseau uniquement) ";
  const secret = document.createElement("input");
  secret.type = "password";
  secret.autocomplete = "off";
  secret.spellcheck = false;
  keyLabel.append(secret);
  keyForm.append(keyLabel);
  async function keyAction(command: string, args?: Record<string, unknown>) {
    try { await invoke(command, args); show("Opération trousseau effectuée. Aucune valeur renvoyée."); }
    catch { show("Échec du trousseau ; aucun stockage de substitution."); }
  }
  button(keyForm, "Enregistrer la clé", "key-save", () => {
    const value = secret.value;
    secret.value = "";
    void keyAction("prototype_set_key", { secret: value });
  });
  button(keyForm, "Supprimer la clé", "key-delete", () => void keyAction("prototype_delete_key"));
  button(keyForm, "Vérifier présence", "key-presence", () => {
    void invoke<boolean>("prototype_key_presence").then(present => show(present ? "Clé présente au trousseau." : "Aucune clé au trousseau."))
      .catch(() => show("Échec du trousseau ; présence inconnue."));
  });
  root.replaceChildren(heading, menu, toolbar, status, notice, editor, recovered, keyForm);
  show();
  // Internal polling makes a blocked native close visible without requiring event permissions.
  const timer = window.setInterval(() => {
    if (acting || sending || queue.length || disposed) return;
    const observed = canonical;
    void invoke<Snapshot>("prototype_snapshot").then(s => {
      // An older poll cannot undo a later acknowledged transaction or mode change.
      if (disposed || acting || sending || queue.length || canonical !== observed) return;
      if (s.noteId < canonical.noteId || s.manualPeriod < canonical.manualPeriod
        || (s.noteId === canonical.noteId && s.revision < canonical.revision)) return;
      if (!synchronized || s.noteId !== canonical.noteId || s.manualPeriod !== canonical.manualPeriod || s.revision !== canonical.revision) {
        const periodEnded = s.noteId !== canonical.noteId || s.manualPeriod !== canonical.manualPeriod;
        if (periodEnded && !s.closePending) recovery = "";
        const recoveryMessage = periodEnded && s.closePending
          ? "Historique réinitialisé ; saisie non acceptée récupérable avant fermeture."
          : periodEnded ? "Historique et saisie non acceptée de la période précédente supprimés."
            : "Historique réinitialisé ; saisie non acceptée récupérable sans rejeu.";
        restore(s, `${s.message} ${recoveryMessage}`);
      } else {
        if (s.message !== canonical.message) feedback = s.message;
        canonical = s; show();
      }
    }).catch(() => {
      if (disposed || canonical !== observed) return;
      synchronized = false;
      recovery = projected.source;
      recovered.textContent = recovery;
      recovered.hidden = !recovery;
      view.setProps({ editable: () => false });
      show("Échec de synchronisation : édition suspendue, saisie récupérable en mémoire ; aucun rejeu.");
    });
  }, 1000);
  return {
    view,
    destroy() {
      disposed = true;
      closeMenu();
      menu.removeEventListener("click", menuClick);
      menu.removeEventListener("keydown", menuKey);
      menu.removeEventListener("focusout", leaveMenu);
      document.removeEventListener("click", outsideClick);
      document.removeEventListener("keydown", escapeMenu);
      window.clearInterval(timer);
      secret.value = ""; recovery = ""; queue.length = 0; view.destroy();
    },
  };
}
