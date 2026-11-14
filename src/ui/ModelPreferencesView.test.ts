// @vitest-environment jsdom
import { describe,it,expect,vi } from "vitest";
import { createModelPreferencesView } from "./ModelPreferencesView";
import { mountPreferences } from "./Preferences";
const settings={ stt:{provider:"openai",model:"gpt-realtime-whisper"},llm:{provider:"openai",model:"gpt-6-luna"},lastNote:null };
describe("préférences de modèles",()=>{
 it("ferme par Echap après une erreur ayant déplacé le focus sur le document",async()=>{
  const root=document.createElement("main");document.body.replaceChildren(root);
  const invoke=vi.fn(async(command:string)=>{
   if(command==="preferences_get")return settings;
   if(command==="preferences_set")throw new Error("synthetic write failure");
  });
  const view=await mountPreferences(root,invoke);
  try{
   const save=root.querySelector<HTMLButtonElement>("[data-action=models-save]")!;
   save.focus();save.click();
   await vi.waitFor(()=>expect(root.textContent).toContain("Enregistrement impossible"));
   document.body.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true,cancelable:true}));
   await vi.waitFor(()=>expect(invoke).toHaveBeenCalledWith("preferences_close"));
   expect(invoke.mock.calls.filter(call=>call[0]==="preferences_close")).toHaveLength(1);
  }finally{view.destroy();}
 });
 it("exécute dix parcours de brouillon OK/Annuler/Echap/échec sans fermer avant persistance",async()=>{
  for(let index=0;index<10;index++){
   const close=vi.fn(async()=>{});
   let release!:()=>void;
   const invoke=vi.fn(async(command:string)=>{
    if(command==="preferences_get")return settings;
    await new Promise<void>(resolve=>{release=resolve;});
    if(index%5===4)throw new Error("private persistence details");
    return settings;
   });
   const panel=createModelPreferencesView(invoke,undefined,close);
   document.body.replaceChildren(panel.element);
   await panel.toggle();
   panel.element.querySelector<HTMLInputElement>('[data-model=llm]')!.value="draft-only";
   if(index%5===0){
    panel.element.querySelector<HTMLButtonElement>('[data-action=models-cancel]')!.click();
    await vi.waitFor(()=>expect(close).toHaveBeenCalledOnce());
    expect(invoke.mock.calls.map(call=>call[0])).toEqual(["preferences_get"]);
   }else if(index%5===1){
    panel.element.dispatchEvent(new KeyboardEvent("keydown",{key:"Escape",bubbles:true,cancelable:true}));
    await vi.waitFor(()=>expect(close).toHaveBeenCalledOnce());
    expect(invoke.mock.calls.map(call=>call[0])).toEqual(["preferences_get"]);
   }else{
    panel.element.querySelector<HTMLButtonElement>('[data-action=models-save]')!.click();
    await vi.waitFor(()=>expect(release).toBeTypeOf("function"));
    expect(close).not.toHaveBeenCalled();
    expect(panel.element.hidden).toBe(false);
    release();
    if(index%5===4){
     await vi.waitFor(()=>expect(panel.element.textContent).toContain("Enregistrement impossible"));
     expect(close).not.toHaveBeenCalled();
     expect(panel.element.querySelector<HTMLInputElement>('[data-model=llm]')!.value).toBe("draft-only");
     expect(panel.element.textContent).not.toContain("private persistence details");
    }else await vi.waitFor(()=>expect(close).toHaveBeenCalledOnce());
   }
  }
 });
 it("charge sur ouverture explicite et conserve des choix STT/LLM séparés",async()=>{
  const invoke=vi.fn(async(_command: string)=>settings);const panel=createModelPreferencesView(invoke);
  document.body.replaceChildren(panel.element);
  expect(invoke).not.toHaveBeenCalled();await panel.toggle();
  expect(invoke).toHaveBeenCalledExactlyOnceWith("preferences_get");
  const fields=panel.element.querySelectorAll<HTMLInputElement>('input[data-model]');
  expect(fields).toHaveLength(2);expect(fields[0]!.value).toBe(settings.stt.model);
  expect(fields[1]!.value).toBe(settings.llm.model);
  expect(panel.element.textContent).toContain("OpenAI");
  expect([...panel.element.querySelectorAll("datalist option")].map(e => (e as HTMLOptionElement).value)).toEqual(["gpt-realtime-whisper", "gpt-6-luna"]);
  fields[1]!.value="custom-test-unverified";
  panel.element.querySelector<HTMLButtonElement>('[data-action="models-save"]')!.click();
  await vi.waitFor(()=>expect(invoke).toHaveBeenCalledWith("preferences_set",{models:{stt:settings.stt,llm:{provider:"openai",model:"custom-test-unverified"}}}));
  expect(invoke.mock.calls.map(call => call[0])).toEqual(["preferences_get", "preferences_set"]);
  expect(panel.element.textContent).toContain("prochaine session");
 });
 it("expurge une erreur et ne prétend pas vérifier un compte ou une clé",async()=>{
  const panel=createModelPreferencesView(vi.fn(async()=>{throw new Error("synthetic-private-detail");}));
  await panel.toggle();expect(panel.element.textContent).toContain("indisponibles");
  expect(panel.element.textContent).not.toContain("synthetic-private-detail");
  expect(panel.element.querySelector<HTMLButtonElement>('[data-action="models-save"]')!.disabled).toBe(true);
 });
});
