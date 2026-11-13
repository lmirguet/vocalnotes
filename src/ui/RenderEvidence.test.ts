// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { createRenderEvidence, type RenderEvidence } from "./RenderEvidence";
import { createTranscriptView } from "./TranscriptView";
import { mountDesktopShell } from "./DesktopShell";
import type { Snapshot } from "./contracts";

afterEach(() => { vi.restoreAllMocks(); vi.useRealTimers(); document.body.replaceChildren(); });

function frames() {
  const callbacks = new Map<number, FrameRequestCallback>();
  let serial = 0;
  vi.spyOn(window, "requestAnimationFrame").mockImplementation(callback => {
    callbacks.set(++serial, callback); return serial;
  });
  vi.spyOn(window, "cancelAnimationFrame").mockImplementation(id => { callbacks.delete(id); });
  vi.spyOn(document, "visibilityState", "get").mockReturnValue("visible");
  return () => {
    const batch = [...callbacks.values()]; callbacks.clear();
    batch.forEach(callback => callback(performance.now()));
  };
}
const base: Snapshot = { noteId:1, sessionId:2, generation:3, manualPeriod:1, revision:0,
  dictationMode:"content", context:"", instructions:"", dictationTransitionPending:false, inputGeneration:1,
  mode:"voice", source:"Note", blocks:[{from:0,to:4,raw:"Note",kind:"paragraph"}],
  dirty:false, message:"", transcriptEnabled:true, transcriptPassageId:1,
  transcriptProvisional:"Bonjour", speechStartedAtEpochUs:1_000_000 };

it("émet le premier partiel visible après deux frames, une fois, sans contenu", () => {
  const frame = frames();
  const view = createTranscriptView(vi.fn());
  document.body.append(view.element);
  const events: RenderEvidence[] = [];
  view.element.addEventListener("vocalnotes-render", event => events.push((event as CustomEvent<RenderEvidence>).detail));
  view.render(base);
  expect(events).toHaveLength(0); frame(); expect(events).toHaveLength(0); frame();
  expect(events).toEqual([{ type:"transcript-partial", noteId:1, sessionId:2, generation:3,
    passageId:1, speechStartedAtEpochUs:1_000_000,
    speechEndedAtEpochUs:null, speechToRenderMs:expect.any(Number),
    speechEndToRenderMs:null, timingValid:true, renderedAt:expect.any(Number) }]);
  expect(events[0]!.renderedAt).toBeGreaterThanOrEqual(performance.timeOrigin);
  expect(events[0]!.speechToRenderMs).toBeCloseTo(events[0]!.renderedAt - 1000, 3);
  view.render({...base, transcriptProvisional:"Bonjour suite"}); frame(); frame();
  expect(events).toHaveLength(1);
  view.destroy();
});

it("ignore les générations obsolètes et annule les frames sur clear/destroy ou partiel vide", () => {
  const frame = frames();
  const view = createTranscriptView(vi.fn()); document.body.append(view.element);
  const event = vi.fn(); view.element.addEventListener("vocalnotes-render", event);
  view.render(base); frame(); view.clear(); frame(); expect(event).not.toHaveBeenCalled();
  view.render({...base, generation:4}); frame();
  view.render({...base, generation:3, transcriptProvisional:"obsolète"});
  expect(view.element.textContent).not.toContain("obsolète");
  frame(); expect(event).toHaveBeenCalledTimes(1);
  view.render({...base, generation:4, transcriptPassageId:2}); frame();
  view.render({...base, generation:4, transcriptPassageId:2, transcriptProvisional:""});
  frame(); expect(event).toHaveBeenCalledTimes(1);
  view.render({...base, generation:4, transcriptPassageId:3}); frame();
  view.destroy(); frame(); expect(event).toHaveBeenCalledTimes(1);
});

it("n'annonce aucun rendu déconnecté ou masqué et annule une génération remplacée", () => {
  const frame = frames(); const target = document.createElement("div");
  const evidence = createRenderEvidence(target, "note-commit"); const event = vi.fn();
  target.addEventListener("vocalnotes-render", event);
  evidence.queue(base, 1, () => true); frame(); frame(); expect(event).not.toHaveBeenCalled();
  document.body.append(target); target.hidden = true;
  evidence.queue(base, 1, () => true); frame(); frame(); expect(event).not.toHaveBeenCalled();
  target.hidden = false;
  evidence.queue(base, 1, () => true); frame();
  evidence.queue({...base, generation:4}, 2, () => true); frame(); frame();
  expect(event).toHaveBeenCalledTimes(1);
  expect((event.mock.calls[0]![0] as CustomEvent<RenderEvidence>).detail).toMatchObject({generation:4,passageId:2});
  evidence.destroy();
});

it("signale des horodatages invalides sans publier NaN ni délais négatifs", () => {
  const frame = frames(); const target = document.createElement("div");
  document.body.append(target);
  const evidence = createRenderEvidence(target, "note-commit");
  const events: RenderEvidence[] = [];
  target.addEventListener("vocalnotes-render", event => events.push((event as CustomEvent<RenderEvidence>).detail));
  evidence.queue({...base, appliedSpeechStartedAtEpochUs:NaN,
    appliedSpeechEndedAtEpochUs:Number.MAX_SAFE_INTEGER}, 1, () => true);
  frame(); frame();
  expect(events).toHaveLength(1);
  expect(events[0]).toMatchObject({
    speechStartedAtEpochUs:null,
    speechEndedAtEpochUs:Number.MAX_SAFE_INTEGER,
    speechToRenderMs:null,
    speechEndToRenderMs:null,
    timingValid:false,
  });
  evidence.queue({...base, appliedSpeechStartedAtEpochUs:2_000_000,
    appliedSpeechEndedAtEpochUs:1_000_000}, 2, () => true);
  frame(); frame();
  expect(events[1]).toMatchObject({
    speechStartedAtEpochUs:2_000_000,
    speechEndedAtEpochUs:1_000_000,
    speechToRenderMs:null,
    speechEndToRenderMs:null,
    timingValid:false,
  });
  evidence.destroy();
});

it("corrèle le rendu de la note validée sans émission après destruction", async () => {
  vi.useFakeTimers(); const frame = frames();
  let snapshot = {...base, mode:"manual", transcriptEnabled:false, appliedPassageId:1,
    appliedSpeechStartedAtEpochUs:1_000_000, appliedSpeechEndedAtEpochUs:2_000_000};
  const root = document.createElement("main"); document.body.append(root);
  const events: RenderEvidence[] = [];
  root.addEventListener("vocalnotes-render", event => events.push((event as CustomEvent<RenderEvidence>).detail));
  const app = await mountDesktopShell(root, async command => {
    if (command === "preferences_get") return {stt:{model:"stt"},llm:{model:"llm"}};
    return snapshot;
  });
  expect(events).toHaveLength(0); frame(); expect(events).toHaveLength(0); frame();
  expect(events).toEqual([{type:"note-commit",noteId:1,sessionId:2,generation:3,passageId:1,
    speechStartedAtEpochUs:1_000_000, speechEndedAtEpochUs:2_000_000,
    speechToRenderMs:expect.any(Number), speechEndToRenderMs:expect.any(Number),
    timingValid:true,
    renderedAt:expect.any(Number)}]);
  expect(events[0]!.speechEndToRenderMs).toBeCloseTo(events[0]!.renderedAt - 2000, 3);
  snapshot = {...snapshot, revision:1, appliedPassageId:2, source:"Suite",
    blocks:[{from:0,to:5,raw:"Suite",kind:"paragraph"}]};
  await vi.advanceTimersByTimeAsync(1000);
  expect(app.view.state.doc.textContent).toBe("Suite");
  frame(); app.destroy(); frame();
  expect(events).toHaveLength(1);
});
