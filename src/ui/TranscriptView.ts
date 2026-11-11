import type { Snapshot } from "./contracts";
import { createRenderEvidence } from "./RenderEvidence";

// Session memory only: never localStorage, console, or editor content.
export function createTranscriptView(change: (enabled: boolean) => void) {
  const element = document.createElement("section");
  element.setAttribute("aria-label", "Transcription temporaire");
  const label = document.createElement("label");
  const toggle = document.createElement("input");
  toggle.type = "checkbox"; toggle.dataset.transcript = "enabled";
  label.append(toggle, document.createTextNode("Afficher la transcription temporaire pour cette écoute"));
  const warning = document.createElement("p");
  warning.textContent = "Contenu potentiellement sensible et inexact. Affichage en mémoire uniquement, effacé à la pause, à l’arrêt ou à la désactivation. Aucun enregistrement audio.";
  const provisional = document.createElement("pre");
  const evidence = createRenderEvidence(provisional, "transcript-partial");
  provisional.setAttribute("aria-label", "Transcription provisoire — susceptible de changer");
  const final = document.createElement("pre");
  final.setAttribute("aria-label", "Dernière transcription finalisée — avant interprétation");
  const provisionalLabel = document.createElement("p"); provisionalLabel.textContent = "Provisoire — susceptible de changer";
  const finalLabel = document.createElement("p"); finalLabel.textContent = "Dernier passage finalisé — avant interprétation";
  const body = document.createElement("div");
  body.append(provisionalLabel, provisional, finalLabel, final); body.hidden = true;
  toggle.addEventListener("change", () => change(toggle.checked));
  element.append(label, warning, body);
  function clear() { evidence.cancel(); toggle.checked = false; provisional.textContent = final.textContent = ""; body.hidden = true; }
  return { element, clear, destroy() { clear(); evidence.destroy(); }, render(snapshot: Snapshot) {
    if (snapshot.sessionId !== undefined && !evidence.accept(snapshot)) return;
    const enabled = snapshot.mode === "voice" && snapshot.transcriptEnabled === true;
    toggle.disabled = snapshot.mode !== "voice";
    if (!enabled) { clear(); return; }
    toggle.checked = true; body.hidden = false;
    provisional.textContent = snapshot.transcriptProvisional ?? "";
    final.textContent = snapshot.transcriptFinal ?? "";
    if (!provisional.textContent.trim()) evidence.cancel();
    else evidence.queue(snapshot, snapshot.transcriptPassageId, () => !!provisional.textContent?.trim());
  } };
}
