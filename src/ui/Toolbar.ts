import type { Command } from "prosemirror-state";
import { toggleMark, setBlockType } from "prosemirror-commands";
import { wrapInList } from "prosemirror-schema-list";
import { schema } from "./NoteView";

export function button(parent: HTMLElement, label: string, action: string, run: () => void) {
  const b = document.createElement("button");
  b.type = "button"; b.textContent = label; b.dataset.action = action;
  b.addEventListener("click", run);
  b.addEventListener("mousedown", event => event.preventDefault());
  parent.append(b); return b;
}

type IconName = "content" | "context" | "instructions" | "bold" | "italic" | "heading" | "paragraph" | "list";
const svgNamespace = "http://www.w3.org/2000/svg";

function icon(name: IconName, mode: boolean) {
  const svg = document.createElementNS(svgNamespace, "svg");
  svg.setAttribute("viewBox", "0 0 16 16");
  svg.setAttribute("aria-hidden", "true");
  svg.classList.add(mode ? "mode-icon" : "format-icon");
  const path = (d: string) => {
    const shape = document.createElementNS(svgNamespace, "path");
    shape.setAttribute("d", d);
    svg.append(shape);
  };
  if (name === "content") {
    path("M3 1.5h7l3 3v10H3z M10 1.5v3h3 M5.5 7h5 M5.5 9.5h5 M5.5 12h3");
  } else if (name === "context") {
    path("M2 2.5h12v11H2z M4.5 5.5h7 M4.5 8h7 M4.5 10.5h4");
  } else if (name === "instructions") {
    path("M2 2.5h12v8H8l-3 3v-3H2z M8 4.5v3 M8 9h.01");
  } else if (name === "list") {
    path("M5 3.5h8 M5 8h8 M5 12.5h8 M2 3.5h.01 M2 8h.01 M2 12.5h.01");
  } else {
    const label = name === "bold" ? "B" : name === "italic" ? "I" : name === "heading" ? "H1" : "¶";
    const text = document.createElementNS(svgNamespace, "text");
    text.textContent = label;
    text.setAttribute("x", name === "heading" ? "1" : "3");
    text.setAttribute("y", "12");
    text.setAttribute("font-size", name === "heading" ? "9" : "13");
    if (name === "bold") text.setAttribute("font-weight", "700");
    if (name === "italic") text.setAttribute("font-style", "italic");
    svg.append(text);
  }
  return svg;
}

function iconButton(parent: HTMLElement, label: string, action: string, iconName: IconName,
  mode: boolean, tooltip: string, run: () => void) {
  const control = document.createElement("button");
  control.type = "button";
  control.dataset.action = action;
  control.classList.add(mode ? "mode-button" : "format-button");
  control.setAttribute("aria-label", label);
  control.title = tooltip;
  control.append(icon(iconName, mode), document.createTextNode(label));
  control.addEventListener("click", run);
  control.addEventListener("mousedown", event => event.preventDefault());
  parent.append(control);
  return control;
}

export function createToolbar(actions: {
  format: (command: Command) => void;
  act: (action: Record<string, string | boolean>) => void;
  reformat: () => void;
}) {
  const element = document.createElement("div");
  element.className = "toolbar-stack";
  const modeToolbar = document.createElement("div");
  modeToolbar.setAttribute("role", "toolbar");
  modeToolbar.setAttribute("aria-label", "Modes et reformulation");
  const formattingToolbar = document.createElement("div");
  formattingToolbar.setAttribute("role", "toolbar");
  formattingToolbar.setAttribute("aria-label", "Mise en forme");
  const modeGroup = document.createElement("div");
  modeGroup.setAttribute("role", "group");
  modeGroup.setAttribute("aria-label", "Mode de dictée");
  const modeButtons = new Map<string, HTMLButtonElement>();
  let instructionAnchor: HTMLSpanElement | undefined;
  let instructionButton: HTMLButtonElement | undefined;
  let instructionReformatButton: HTMLButtonElement | undefined;
  for (const [mode, label, iconName, tooltip] of [
    ["content", "Contenu", "content", "Démarrer la dictée dans le contenu ; recliquer pour arrêter"],
    ["context", "Contexte", "context", "Démarrer la dictée dans le contexte ; recliquer pour arrêter"],
    ["instructions", "Instruction", "instructions", "Dicter une instruction temporaire ; recliquer pour arrêter"],
  ] as const) {
    let parent: HTMLElement = modeGroup;
    if (mode === "instructions") {
      instructionAnchor = document.createElement("span");
      instructionAnchor.className = "instruction-mode-anchor";
      parent = instructionAnchor;
      modeGroup.append(instructionAnchor);
    }
    const control = iconButton(parent, label, `dictation-${mode}`, iconName, true, tooltip, () =>
      actions.act({ type: "set_dictation_mode", mode }));
    control.dataset.dictationMode = mode;
    control.setAttribute("aria-pressed", "false");
    modeButtons.set(mode, control);
    if (mode === "instructions") instructionButton = control;
  }
  const pendingMode = document.createElement("span");
  pendingMode.dataset.dictationPending = "true";
  pendingMode.setAttribute("aria-live", "polite");
  pendingMode.textContent = "Sortie d'Instructions différée : transcription en attente.";
  pendingMode.hidden = true;
  const reformatButton = button(modeToolbar, "Reformuler le contenu", "reformat", actions.reformat);
  const reformatStatus = document.createElement("span");
  reformatStatus.dataset.reformatStatus = "true";
  reformatStatus.setAttribute("aria-live", "polite");
  reformatStatus.hidden = true;
  modeToolbar.append(modeGroup, pendingMode, reformatStatus);
  element.append(modeToolbar, formattingToolbar);
  const formatting = [
    ["Gras", "Gras", "bold", toggleMark(schema.marks.strong!)],
    ["Italique", "Italique", "italic", toggleMark(schema.marks.em!)],
    ["Titre", "Titre", "heading", setBlockType(schema.nodes.heading!, { level: 1 })],
    ["Paragraphe", "Paragraphe", "paragraph", setBlockType(schema.nodes.paragraph!)],
    ["Liste à puces", "list", "list", wrapInList(schema.nodes.bullet_list!)],
  ] as const;
  const buttons = formatting.map(([label, action, iconName, command]) =>
    iconButton(formattingToolbar, label, action, iconName, false, label, () => actions.format(command)));
  return { element,
    setFormattingEnabled(enabled: boolean) { for (const b of buttons) b.disabled = !enabled; },
    setDictationMode(mode: string | null, pending: boolean) {
      for (const [key, control] of modeButtons) control.setAttribute("aria-pressed", String(key === mode));
      instructionButton?.setAttribute("aria-expanded", String(mode === "instructions"));
      if (instructionAnchor?.lastElementChild instanceof HTMLElement
        && instructionAnchor.lastElementChild.classList.contains("instruction-popup")) {
        instructionAnchor.lastElementChild.hidden = mode !== "instructions";
      }
      pendingMode.hidden = !pending;
    },
    setInstructionPopup(popup: HTMLElement) {
      popup.classList.add("instruction-popup");
      instructionButton?.setAttribute("aria-controls", popup.id);
      popup.hidden = true;
      instructionReformatButton = button(
        popup,
        "Valider les instructions",
        "instruction-reformat",
        actions.reformat,
      );
      instructionReformatButton.setAttribute("aria-label", "Valider les instructions et reformuler le contenu");
      instructionReformatButton.title = "Envoyer le contexte, les instructions et le contenu pour reformuler la note";
      instructionReformatButton.classList.add("instruction-reformat");
      instructionAnchor?.append(popup);
    },
    setReformatBusy(busy: boolean) {
      reformatButton.disabled = busy;
      reformatButton.setAttribute("aria-busy", String(busy));
      if (instructionReformatButton) {
        instructionReformatButton.disabled = busy;
        instructionReformatButton.setAttribute("aria-busy", String(busy));
      }
      reformatStatus.hidden = !busy;
      reformatStatus.textContent = busy ? "Reformulation en cours ; vous pouvez continuer à modifier la note." : "";
    },
  };
}
