// @vitest-environment jsdom
import { expect, it, vi } from "vitest";
import { createTranscriptView } from "./TranscriptView";
import type { Snapshot } from "./contracts";

it("affiche le texte comme données et purge la vue quand la session finit", () => {
  const change = vi.fn();
  const view = createTranscriptView(change);
  const base = { mode: "voice", transcriptEnabled: true,
    transcriptProvisional: "provisoire", transcriptFinal: '<img src=x onerror="alert(1)">' } as Snapshot;
  view.render(base);
  expect(view.element.querySelector("img")).toBeNull();
  expect(view.element.textContent).toContain(base.transcriptFinal);
  const toggle = view.element.querySelector("input")!;
  toggle.checked = false; toggle.dispatchEvent(new Event("change"));
  expect(change).toHaveBeenCalledWith(false);
  view.render({ ...base, mode: "paused" });
  expect(toggle.checked).toBe(false);
  expect(view.element.textContent).not.toContain("<img");
  expect(view.element.textContent).not.toContain("provisoire");
  expect(toggle.disabled).toBe(true);
  view.render({ ...base, transcriptEnabled: false });
  expect(view.element.querySelector("pre")!.textContent).toBe("");
});

it("rend les deltas avant le final sans concaténer une seconde fois le texte complet", () => {
  const view = createTranscriptView(vi.fn());
  const base = { mode: "voice", transcriptEnabled: true,
    transcriptProvisional: "Hello", transcriptFinal: "" } as Snapshot;
  const [provisional, final] = view.element.querySelectorAll("pre");
  view.render(base);
  expect(provisional!.textContent).toBe("Hello");
  expect(final!.textContent).toBe("");
  view.render({ ...base, transcriptProvisional: "Hello world" });
  expect(provisional!.textContent).toBe("Hello world");
  view.render({ ...base, transcriptProvisional: "", transcriptFinal: "Hello world." });
  expect(provisional!.textContent).toBe("");
  expect(final!.textContent).toBe("Hello world.");
  view.render({ ...base, transcriptProvisional: "Bonjour", transcriptFinal: "Hello world." });
  expect(provisional!.textContent).toBe("Bonjour");
  view.render({ ...base, mode: "manual" });
  expect(provisional!.textContent).toBe("");
  expect(final!.textContent).toBe("");
});
