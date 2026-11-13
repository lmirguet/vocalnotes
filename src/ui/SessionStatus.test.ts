// @vitest-environment jsdom
import { expect, it } from "vitest";
import { createSessionStatus } from "./SessionStatus";
import type { Snapshot } from "./contracts";

it("annonce les états réels du microphone et la cible active sans commande ni changement de focus", () => {
  const status = createSessionStatus();
  const input = document.createElement("input"); document.body.replaceChildren(input, status.element); input.focus();
  const base = { mode: "voice", dictationMode: "content", listeningState: "starting", voiceActivity: "idle" } as Snapshot;
  status.render(base);
  expect(status.element.textContent).toContain("Initialisation du microphone");
  expect(status.element.textContent).toContain("mode Contenu");
  expect(status.element.textContent).not.toContain("Écoute active");
  status.render({ ...base, listeningState: "listening" });
  expect(status.element.textContent).toContain("Écoute active");
  for (const [activity, label] of [["processing", "Traitement"], ["clarification", "Précision demandée"], ["error", "Erreur"]] as const) {
    status.render({ ...base, listeningState: "listening", voiceActivity: activity });
    expect(status.element.textContent).toContain(label);
  }
  status.render({ ...base, mode: "manual", listeningState: "stopped", dictationMode: "instructions" });
  expect(status.element.textContent).toContain("Microphone arrêté");
  expect(status.element.textContent).toContain("aucun mode sélectionné");
  expect(status.element.querySelector("button")).toBeNull();
  expect(document.activeElement).toBe(input);
});

it("distingue le modèle STT configuré et acquitté sans les dire servis", () => {
  const status = createSessionStatus();
  const base = { mode: "voice", dictationMode: "content", listeningState: "starting",
    sttConfiguredModel: "configured-stt", sttAcknowledgedModel: "" } as Snapshot;
  status.render(base);
  expect(status.element.textContent).toContain("STT configuré : configured-stt");
  expect(status.element.textContent).not.toContain("STT acquitté");
  expect(status.element.textContent).not.toContain("servi");
  status.render({ ...base, listeningState: "listening", sttAcknowledgedModel: "acknowledged-stt" });
  expect(status.element.textContent).toContain("STT configuré : configured-stt");
  expect(status.element.textContent).toContain("STT acquitté : acknowledged-stt");
  expect(status.element.textContent).not.toContain("servi");
});
