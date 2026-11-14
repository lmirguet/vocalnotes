import type { Invoke } from "./contracts";
import { createModelPreferencesView } from "./ModelPreferencesView";
import { button } from "./Toolbar";

export async function mountPreferences(root: HTMLElement, invoke: Invoke) {
  root.className = "preferences-page";
  const secret = document.createElement("input");
  secret.type = "password"; secret.autocomplete = "off"; secret.spellcheck = false;
  secret.setAttribute("aria-label", "Nouvelle clé OpenAI");
  const view = createModelPreferencesView(invoke, undefined, async () => {
    secret.value = "";
    await invoke("preferences_close");
  });
  const keys = document.createElement("fieldset");
  const title = document.createElement("legend");
  title.textContent = "Trousseau système — actions explicites immédiates";
  const notice = document.createElement("p");
  notice.textContent = "Une clé simplement saisie reste locale au champ. Les boutons ci-dessous modifient immédiatement le trousseau, indépendamment de OK ou Annuler. Aucune clé n'est relue.";
  const status = document.createElement("p");
  status.setAttribute("role", "status");
  keys.append(title, notice, secret);
  async function keyAction(command: string, args?: Record<string, unknown>) {
    try { await invoke(command, args); status.textContent = "Opération trousseau effectuée. Aucune clé renvoyée."; }
    catch { status.textContent = "Trousseau indisponible ; aucun stockage de substitution."; }
  }
  button(keys, "Enregistrer la clé", "key-save", () => {
    const value = secret.value; secret.value = "";
    void keyAction("prototype_set_key", { secret: value });
  });
  button(keys, "Supprimer la clé", "key-delete", () => void keyAction("prototype_delete_key"));
  button(keys, "Vérifier présence", "key-presence", () => {
    void invoke("prototype_key_presence").then(present => {
      status.textContent = present ? "Clé présente au trousseau." : "Aucune clé au trousseau.";
    }).catch(() => { status.textContent = "Présence inconnue ; trousseau indisponible."; });
  });
  keys.append(status);
  // Keep the entire modal form in one keyboard scope.
  view.element.insertBefore(keys, view.element.querySelector('[data-action="models-save"]'));
  root.replaceChildren(view.element);
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && !event.defaultPrevented) {
      event.preventDefault();
      void view.cancel();
      return;
    }
    if (event.key !== "Tab") return;
    const fields = [...root.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled), select:not(:disabled)")];
    const index = fields.indexOf(document.activeElement as HTMLElement);
    if (!fields.length) { event.preventDefault(); return; }
    if (event.shiftKey && index <= 0) { event.preventDefault(); fields.at(-1)!.focus(); }
    else if (!event.shiftKey && (index < 0 || index === fields.length - 1)) { event.preventDefault(); fields[0]!.focus(); }
  };
  root.ownerDocument.addEventListener("keydown", onKeyDown);
  await view.toggle();
  root.querySelector<HTMLElement>("input,select")?.focus();
  return { destroy() {
    root.ownerDocument.removeEventListener("keydown", onKeyDown);
    secret.value = ""; root.replaceChildren();
  } };
}
