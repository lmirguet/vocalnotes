// @vitest-environment jsdom
import { expect, it, vi } from "vitest";
import { createSessionStatus } from "./SessionStatus";
import type { Snapshot } from "./contracts";

it("distingue initialisation, écoute, traitement, précision, erreur et pause sans prendre le focus", () => {
  const pause = vi.fn(); const status = createSessionStatus(pause);
  const input = document.createElement("input"); document.body.replaceChildren(input, status.element); input.focus();
  const base = { mode: "voice", listeningState: "starting", voiceActivity: "idle" } as Snapshot;
  status.render(base);
  expect(status.element.textContent).toContain("Initialisation");
  expect(status.element.textContent).not.toContain("Écoute en cours");
  status.render({ ...base, listeningState: "listening" });
  expect(status.element.textContent).toContain("Écoute en cours");
  expect(status.element.textContent).toContain("tiers");
  status.element.querySelector("button")!.click(); expect(pause).toHaveBeenCalledOnce();
  for (const [activity, label] of [["processing", "Traitement"], ["clarification", "Précision demandée"], ["error", "Erreur"]] as const) {
    status.render({ ...base, listeningState: "listening", voiceActivity: activity });
    expect(status.element.textContent).toContain(label);
  }
  for (const [mode, label] of [["paused", "Pause"], ["manual", "Manuel"]] as const) {
    status.render({ ...base, mode, listeningState: "stopped" });
    expect(status.element.textContent).toContain(label);
    expect(status.element.textContent).toContain("micro arrêté");
    expect(status.element.querySelector("button")!.disabled).toBe(true);
  }
  expect(document.activeElement).toBe(input);
});

it("distingue le modèle STT configuré et le modèle acquitté sans les dire servis", () => {
  const status = createSessionStatus(vi.fn());
  const base = { mode:"voice", listeningState:"starting", sttConfiguredModel:"configured-stt",
    sttAcknowledgedModel:"" } as Snapshot;
  status.render(base);
  expect(status.element.textContent).toContain("STT configuré : configured-stt");
  expect(status.element.textContent).not.toContain("STT acquitté");
  expect(status.element.textContent).not.toContain("servi");
  status.render({ ...base, listeningState:"listening", sttAcknowledgedModel:"acknowledged-stt" });
  expect(status.element.textContent).toContain("STT configuré : configured-stt");
  expect(status.element.textContent).toContain("STT acquitté : acknowledged-stt");
  expect(status.element.textContent).not.toContain("servi");
});
