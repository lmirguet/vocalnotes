// @vitest-environment jsdom
import { describe,it,expect,vi } from "vitest";
import { createModelPreferencesView } from "./ModelPreferencesView";
const settings={ stt:{provider:"mistral",model:"voxtral-mini-transcribe-realtime-2602"},llm:{provider:"mistral",model:"mistral-large-latest"},lastNote:null };
describe("préférences de modèles",()=>{
 it("charge sur ouverture explicite et conserve des choix STT/LLM séparés",async()=>{
  const invoke=vi.fn(async()=>settings);const panel=createModelPreferencesView(invoke);
  document.body.replaceChildren(panel.element);
  expect(invoke).not.toHaveBeenCalled();await panel.toggle();
  expect(invoke).toHaveBeenCalledExactlyOnceWith("preferences_get");
  const fields=panel.element.querySelectorAll<HTMLInputElement>('input[data-model]');
  expect(fields).toHaveLength(2);expect(fields[0]!.value).toBe(settings.stt.model);
  expect(fields[1]!.value).toBe(settings.llm.model);
  expect(panel.element.textContent).not.toContain("OpenAI");
  fields[1]!.value="mistral-test-unverified";
  panel.element.querySelector<HTMLButtonElement>('[data-action="models-save"]')!.click();
  await vi.waitFor(()=>expect(invoke).toHaveBeenCalledWith("preferences_set",{models:{stt:settings.stt,llm:{provider:"mistral",model:"mistral-test-unverified"}}}));
  expect(panel.element.textContent).toContain("prochaine session");
 });
 it("expurge une erreur et ne prétend pas vérifier un compte ou une clé",async()=>{
  const panel=createModelPreferencesView(vi.fn(async()=>{throw new Error("synthetic-private-detail");}));
  await panel.toggle();expect(panel.element.textContent).toContain("indisponibles");
  expect(panel.element.textContent).not.toContain("synthetic-private-detail");
  expect(panel.element.querySelector<HTMLButtonElement>('[data-action="models-save"]')!.disabled).toBe(true);
 });
});
