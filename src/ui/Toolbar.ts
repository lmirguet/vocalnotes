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

export function createToolbar(actions: {
  format: (command: Command) => void;
  act: (action: Record<string, string | boolean>) => void;
  changed: () => void;
}) {
  const element = document.createElement("div");
  element.setAttribute("role", "toolbar"); element.setAttribute("aria-label", "Mise en forme et vocal");
  const formatting = [
    ["Gras", "Gras", toggleMark(schema.marks.strong!)],
    ["Italique", "Italique", toggleMark(schema.marks.em!)],
    ["Titre", "Titre", setBlockType(schema.nodes.heading!, { level: 1 })],
    ["Paragraphe", "Paragraphe", setBlockType(schema.nodes.paragraph!)],
    ["Liste", "list", wrapInList(schema.nodes.bullet_list!)],
  ] as const;
  const buttons = formatting.map(([label, action, command]) => button(element, label, action, () => actions.format(command)));
  const label = document.createElement("label");
  const consent = document.createElement("input"); consent.type = "checkbox"; consent.dataset.consent = "voice";
  label.append(consent, document.createTextNode("J'autorise le micro et les envois Mistral décrits ci-dessus, coût inconnu."));
  element.append(label); consent.addEventListener("change", actions.changed);
  const startButton = button(element, "Démarrer le vocal", "start", () => actions.act({ type: "start", consent: consent.checked, resume: false }));
  const resumeButton = button(element, "Reprendre explicitement", "resume", () => actions.act({ type: "start", consent: consent.checked, resume: true }));
  button(element, "Pause", "pause", () => actions.act({ type: "pause" }));
  button(element, "Arrêter le vocal", "stop", () => actions.act({ type: "stop" }));
  return { element, consent, startButton, resumeButton,
    setFormattingEnabled(enabled: boolean) { for (const b of buttons) b.disabled = !enabled; },
  };
}
