import type { Snapshot } from "./contracts";

// Reads canonical UTF-8 source ranges only. Never invent processing targets,
// touch the selection, activate the window, or scroll before an explicit click.
export function createChangeView(locate: (byteOffset: number) => void) {
  const element = document.createElement("section");
  element.setAttribute("aria-label", "Modifications vocales");
  const activity = document.createElement("p"); activity.setAttribute("aria-live", "polite");
  const changes = document.createElement("div");
  element.append(activity, changes);
  let previous = "";
  return { element, render(snapshot: Snapshot) {
    const message = snapshot.voiceActivity === "processing"
      ? "Traitement en cours — aucune nouvelle modification appliquée."
      : "";
    if (activity.textContent !== message) activity.textContent = message;
    const feedback = snapshot.changeFeedback?.revision === snapshot.revision ? snapshot.changeFeedback : null;
    const key = JSON.stringify([snapshot.noteId, snapshot.revision, feedback]);
    if (key === previous) return;
    previous = key;
    changes.replaceChildren();
    if (!feedback?.areas.length) return;
    const heading = document.createElement("p"); heading.textContent = "Dernière modification vocale appliquée :";
    const list = document.createElement("ul");
    const source = new TextEncoder().encode(snapshot.source);
    const decode = (from: number, to: number) => new TextDecoder("utf-8", { fatal: true }).decode(source.slice(from, to));
    for (const area of feedback.areas) {
      if (!Number.isInteger(area.from) || !Number.isInteger(area.to) || area.from < 0 || area.to < area.from || area.to > source.length) continue;
      let text: string;
      try { text = decode(area.from, area.to); } catch { continue; }
      const removed = area.from === area.to;
      const label = area.kind === "move" ? `Déplacement — ${removed ? "retrait" : "insertion"}`
        : area.kind === "delete" ? `Suppression de ${area.removedCharacters} caractères`
          : area.kind === "replace" ? "Remplacement" : "Ajout";
      const row = document.createElement("li");
      row.append(document.createTextNode(`${label} : `));
      const preview = document.createElement("code");
      // Deletion has no surviving text: show neighbouring current text, clearly labelled.
      if (removed) {
        const rest = new TextDecoder().decode(source.slice(area.from));
        const before = new TextDecoder().decode(source.slice(0, area.from));
        preview.textContent = `repère après retrait : ${rest.slice(0, 80) || before.slice(-80) || "note vide"}`;
      } else preview.textContent = text.length > 240 ? `${text.slice(0, 240)}…` : text;
      const button = document.createElement("button"); button.type = "button";
      button.textContent = "Repérer dans la note";
      button.addEventListener("click", () => locate(area.from));
      row.append(preview, document.createTextNode(" "), button); list.append(row);
    }
    if (list.childElementCount) changes.append(heading, list);
  } };
}
