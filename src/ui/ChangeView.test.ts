// @vitest-environment jsdom
import { expect, it, vi } from "vitest";
import { createChangeView } from "./ChangeView";
import type { Snapshot } from "./contracts";

it("ne crée aucune zone pendant le traitement et repère uniquement un commit de la révision courante", () => {
  const locate = vi.fn(); const view = createChangeView(locate);
  const focused = document.createElement("input"); document.body.replaceChildren(focused, view.element); focused.focus();
  const base = { noteId: 1, revision: 2, source: "Été 13.", voiceActivity: "processing" } as Snapshot;
  view.render(base);
  expect(view.element.textContent).toContain("Traitement");
  expect(view.element.querySelector("button")).toBeNull();
  view.render({ ...base, voiceActivity: "idle", changeFeedback: { revision: 2, areas: [{ from: 6, to: 8, kind: "replace", removedCharacters: 2 }] } });
  expect(view.element.textContent).toContain("Remplacement");
  expect(view.element.querySelector("code")?.textContent).toBe("13");
  expect(document.activeElement).toBe(focused);
  view.element.querySelector("button")!.click(); expect(locate).toHaveBeenCalledWith(6);
  view.render({ ...base, changeFeedback: { revision: 1, areas: [{ from: 0, to: 3, kind: "insert", removedCharacters: 0 }] } });
  expect(view.element.querySelector("button")).toBeNull();
});

it("signale suppression et déplacement en texte, sans HTML actif ni déplacement automatique", () => {
  const locate = vi.fn(); const view = createChangeView(locate);
  view.render({ noteId: 1, revision: 1, source: "<img src=x>", changeFeedback: { revision: 1, areas: [
    { from: 0, to: 0, kind: "delete", removedCharacters: 4 },
    { from: 0, to: 0, kind: "move", removedCharacters: 3 },
    { from: 0, to: 11, kind: "move", removedCharacters: 0 },
  ] } } as Snapshot);
  expect(view.element.textContent).toContain("Suppression de 4 caractères");
  expect(view.element.textContent).toContain("Déplacement — retrait");
  expect(view.element.textContent).toContain("Déplacement — insertion");
  expect(view.element.querySelector("img")).toBeNull();
  expect(locate).not.toHaveBeenCalled();
});
