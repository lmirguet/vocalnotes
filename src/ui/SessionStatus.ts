import type { Snapshot } from "./contracts";

const targets: Record<Snapshot["dictationMode"], string> = {
  content: "Contenu",
  context: "Contexte",
  instructions: "Instruction",
};

export function createSessionStatus() {
  const element = document.createElement("section");
  element.className = "session-status";
  element.setAttribute("aria-label", "État de l’écoute");
  const state = document.createElement("p");
  state.setAttribute("aria-live", "polite"); state.setAttribute("aria-atomic", "true");
  element.append(state);
  return { element, render(snapshot: Snapshot) {
    const voice = snapshot.mode === "voice";
    const capture = !voice
      ? "Microphone arrêté"
      : snapshot.listeningState === "listening" ? "Écoute active"
        : snapshot.listeningState === "stopped" ? "Écoute arrêtée" : "Initialisation du microphone";
    const activity = snapshot.voiceActivity === "processing" ? "Traitement du passage finalisé"
      : snapshot.voiceActivity === "clarification" ? "Précision demandée — consulter le message"
        : snapshot.voiceActivity === "error" ? "Erreur — consulter le message et le bilan vocal" : "";
    const target = voice ? `mode ${targets[snapshot.dictationMode]}` : "aucun mode sélectionné";
    const text = `${capture} — ${target}`
      + (activity ? ` — ${activity}` : "")
      + (snapshot.sttConfiguredModel ? ` — STT configuré : ${snapshot.sttConfiguredModel}` : "")
      + (snapshot.sttAcknowledgedModel ? ` — STT acquitté : ${snapshot.sttAcknowledgedModel}` : "");
    if (state.textContent !== text) state.textContent = text;
  } };
}
