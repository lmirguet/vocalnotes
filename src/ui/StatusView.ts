import type { Snapshot } from "./contracts";

export function createStatusView() {
  const status = document.createElement("p");
  status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite");
  status.setAttribute("aria-atomic", "true");
  return status;
}

// State text never creates processing targets and never changes keyboard focus.
export function renderStatus(status: HTMLElement, canonical: Snapshot, projected: Snapshot,
  message: string, pending: number) {
    const mode = projected.mode === "manual" ? "Manuel" : projected.mode === "paused" ? "Pause" : "Vocal";
    const storage = projected.storageMessage ?? "";
    const text = (storage === message ? mode : `${mode} — ${message}`) +
      ` — tentatives STT : ${canonical.sttRequests ?? 0}, LLM : ${canonical.llmRequests ?? 0}, priorité : ${canonical.priorityRequests ?? 0}` +
      ` ; audio transmis : ${canonical.audioMs ?? 0} ms ; coût inconnu.` +
      (canonical.sttModel ? ` STT servi : ${canonical.sttModel}.` : "") +
      (canonical.llmModel ? ` LLM servi : ${canonical.llmModel}.` : "") +
      ` Bilan vocal : ${canonical.voiceApplied ?? 0} modification(s), ${canonical.voiceClarifications ?? 0} clarification(s), ` +
      `${canonical.voiceRejected ?? 0} refus, ${canonical.voiceNoChange ?? 0} sans modification. ` +
      (canonical.lastVoiceResult ?? "Aucun résultat vocal traité.") +
      ` Derniers délais : flush→done ${canonical.lastSttFinalMs ?? "non mesuré"} ms ; ` +
      `LLM ${canonical.lastLlmMs ?? "non mesuré"} ms ; priorité ${canonical.lastPriorityMs ?? "non mesuré"} ms ; ` +
      `commit ${canonical.lastCommitUs ?? "non mesuré"} µs.` +
      (pending ? " — projection en attente, non sauvegardée." : canonical.dirty ? " — note modifiée, non sauvegardée." : "");
    if (status.textContent !== storage + text) {
      const details = document.createElement("span");
      details.textContent = text;
      if (storage) {
        const warning = document.createElement("strong");
        warning.dataset.storageWarning = "true";
        warning.textContent = storage;
        status.replaceChildren(warning, details);
      } else status.replaceChildren(details);
    }
}
