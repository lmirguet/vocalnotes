import type { Invoke } from "./contracts";
type Service = { provider: string; model: string };
type Models = { stt: Service; llm: Service };

// No secret value is accepted or held here. Key management keeps its dedicated IPC.
export function createModelPreferencesView(invoke: Invoke) {
  const element = document.createElement("section");
  element.hidden = true; element.setAttribute("aria-label", "Préférences");
  const title = document.createElement("h2"); title.textContent = "Préférences";
  const notice = document.createElement("p");
  notice.textContent = "Les changements s'appliquent à la prochaine session explicitement démarrée ; une reprise conserve les modèles de la session suspendue. Les modèles personnalisés ne sont pas vérifiés auprès du compte. Aucun contact fournisseur lors de l'enregistrement de ces réglages.";
  const fields = document.createElement("div");
  const modelFields = ([
    ["stt", "Transcription", "voxtral-mini-transcribe-realtime-3-0"],
    ["llm", "Interprétation", "mistral-large-latest"],
  ] as const).map(([key, name, builtin]) => {
    const fieldset = document.createElement("fieldset");
    const legend = document.createElement("legend"); legend.textContent = name;
    const providerLabel = document.createElement("label"); providerLabel.textContent = "Fournisseur ";
    const provider = document.createElement("select"); provider.setAttribute("aria-label", `Fournisseur ${name}`);
    const option = document.createElement("option"); option.value = "mistral"; option.textContent = "Mistral";
    provider.append(option); providerLabel.append(provider);
    const label = document.createElement("label"); label.textContent = " Modèle ";
    const input = document.createElement("input"); input.type = "text"; input.maxLength = 128;
    input.dataset.model = key; input.setAttribute("aria-label", `Modèle ${name}`);
    input.spellcheck = false; input.autocomplete = "off";
    const options = document.createElement("datalist"); options.id = `models-${key}-${crypto.randomUUID()}`;
    const integrated = document.createElement("option"); integrated.value = builtin; integrated.label = "Identifiant intégré";
    options.append(integrated); input.setAttribute("list", options.id); label.append(input, options);
    fieldset.append(legend, providerLabel, label); fields.append(fieldset);
    return { key, input, provider };
  });
  const save = document.createElement("button"); save.type = "button"; save.dataset.action = "models-save";
  save.textContent = "Enregistrer les réglages"; save.disabled = true;
  const message = document.createElement("p"); message.setAttribute("role", "status");
  let loaded = false;
  let pending = false;
  function display(value: Models) {
    for (const field of modelFields) { field.input.value = value[field.key].model; field.provider.value = value[field.key].provider; }
  }
  save.addEventListener("click", () => {
    if (!loaded || pending) return;
    const models = Object.fromEntries(modelFields.map(field => [field.key, { provider:field.provider.value, model:field.input.value }]));
    pending = true; save.disabled = true;
    void invoke("preferences_set", { models }).then(value => {
      display(value as Models); message.textContent = "Réglages enregistrés pour la prochaine session. Accès aux modèles non testé.";
    }).catch(() => { message.textContent = "Enregistrement impossible ; réglages précédents conservés. Vérifier les identifiants et le fichier de préférences."; })
      .finally(() => { pending = false; save.disabled = !loaded; });
  });
  element.append(title, notice, fields, save, message);
  return { element, async toggle() {
    element.hidden = !element.hidden;
    if (element.hidden || pending) return;
    pending = true; save.disabled = true;
    try {
      display(await invoke("preferences_get") as Models); loaded = true;
      message.textContent = "Réglages locaux ; la présence d'une clé ne vérifie pas l'accès aux modèles.";
    } catch { loaded = false; message.textContent = "Préférences indisponibles ; aucun réglage de secours appliqué."; }
    finally { pending = false; save.disabled = !loaded; }
  } };
}
