import type { Snapshot } from "./contracts";

// Current canonical capture state, independent of editor projection and feedback messages.
// No window activation, global listener, system notification, or focus change.
export function createSessionStatus(pause: () => void) {
  const element = document.createElement("section");
  element.setAttribute("aria-label", "État de l’écoute");
  const state = document.createElement("p");
  state.setAttribute("aria-live", "polite"); state.setAttribute("aria-atomic", "true");
  const action = document.createElement("button"); action.type = "button";
  action.textContent = "Mettre l’écoute en pause"; action.dataset.action = "session-pause";
  action.addEventListener("mousedown", event => event.preventDefault());
  action.addEventListener("click", pause);
  const warning = document.createElement("p");
  warning.textContent = "Attention : l’écoute continue en arrière-plan et peut capter les paroles de tiers. Pause ou arrêt à la voix, ou bouton Pause dans cette fenêtre. Après une interruption, la reprise est explicite.";
  element.append(state, action, warning);
  return { element, render(snapshot: Snapshot) {
    const voice = snapshot.mode === "voice";
    const capture = !voice
      ? snapshot.mode === "paused" ? "Pause — micro arrêté" : "Manuel — micro arrêté"
      : snapshot.listeningState === "listening" ? "Écoute en cours"
        : "Initialisation — microphone pas encore prêt";
    const activity = snapshot.voiceActivity === "processing" ? "Traitement du passage finalisé"
      : snapshot.voiceActivity === "clarification" ? "Précision demandée — consulter le message"
        : snapshot.voiceActivity === "error" ? "Erreur — consulter le message et le bilan vocal" : "";
    const text = capture + (activity ? ` — ${activity}` : "");
    if (state.textContent !== text) state.textContent = text;
    action.disabled = !voice;
  } };
}
