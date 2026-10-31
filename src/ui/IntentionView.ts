import type { Snapshot } from "./contracts";

// Note metadata, separate from the editor body; this is not a command input.
export function createIntentionView() {
  const element = document.createElement("section");
  element.setAttribute("aria-label", "Intention de rédaction");
  const label = document.createElement("label");
  label.append(document.createTextNode("Intention de rédaction — enregistrée avec la note"));
  const field = document.createElement("textarea"); field.readOnly = true;
  field.rows = 3; field.placeholder = "Aucune intention précisée";
  field.setAttribute("aria-label", "Intention de rédaction courante");
  label.append(field); element.append(label);
  return { element, clear() { field.value = ""; }, render(snapshot: Snapshot) {
    const intention = snapshot.draftingIntention;
    const value = intention ? [
      intention.purpose,
      intention.form && `Forme : ${intention.form}`,
      intention.tone && `Ton : ${intention.tone}`,
      intention.context && `Contexte : ${intention.context}`,
    ].filter(Boolean).join("\n") : "";
    if (field.value !== value) field.value = value;
  } };
}
