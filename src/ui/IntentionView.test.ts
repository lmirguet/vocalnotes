// @vitest-environment jsdom
import { expect, it } from "vitest";
import { createIntentionView } from "./IntentionView";
import type { Snapshot } from "./contracts";

it("affiche le profil courant en lecture seule, conserve le focus et efface le profil purgé", () => {
  const view = createIntentionView();
  const input = document.createElement("input"); document.body.replaceChildren(input, view.element); input.focus();
  const field = view.element.querySelector("textarea")!;
  view.render({ draftingIntention: null } as Snapshot);
  expect(field.value).toBe(""); expect(field.readOnly).toBe(true);
  expect(field.placeholder).toContain("Aucune intention");
  const profile = { purpose: "<img src=x>", form: "e-mail", tone: "cordial", context: "Devis non confirmé" };
  view.render({ mode: "voice", draftingIntention: profile } as Snapshot);
  expect(field.value).toBe("<img src=x>\nForme : e-mail\nTon : cordial\nContexte : Devis non confirmé");
  expect(view.element.querySelector("img")).toBeNull();
  expect(document.activeElement).toBe(input);
  view.render({ mode: "paused", draftingIntention: profile } as Snapshot);
  expect(field.value).toContain("e-mail");
  view.render({ mode: "manual", draftingIntention: profile } as Snapshot);
  expect(field.value).toContain("e-mail");
  view.render({ mode: "manual", draftingIntention: null } as Snapshot);
  expect(field.value).toBe("");
});
