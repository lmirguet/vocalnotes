# UI Contract: Refonte de l'interface desktop Vocalnotes

This document defines the user-visible contract. It does not introduce a public network API. Existing private Tauri commands may be coordinated to implement these transitions.

## Desktop layout

- Use the native Windows title bar; within the client area, present a compact menu bar, a mode/reformat toolbar, a separate formatting toolbar, the note workspace, and a status bar. Do not duplicate window controls in the frontend.
- Place the smaller context panel above the larger Markdown editor with a visible separator.
- Keep the note editor as the dominant work area; do not use a dashboard/card layout.
- Retain existing menu actions, note controls, model/preferences views, and status information without inventing new commands.

## Mode toolbar

- Provide exactly three mode buttons, each with a distinct local icon and visible label: **Contenu**, **Contexte**, and **Instruction**.
- Expose the selected state accessibly (`aria-pressed` or an equivalent native state), and ensure mode buttons remain keyboard reachable.
- On initial application entry, no mode is selected and capture remains stopped.
- Selecting an unselected mode starts capture for that target. Selecting a different active mode drains and ends the prior target before starting the new one.
- Selecting the active mode again stops capture and deselects all modes.
- No separate in-editor microphone permission, start, resume, pause, or stop control is shown.
- Display the active target and actual capture state in the status bar; before a mode is selected, show that the microphone is stopped and no mode is selected. Initialization and failures must not be presented as active listening.

## Instruction popup

- The Instruction button opens a borderless, modeless popup near the mode toolbar.
- The popup contains an accessible editable instructions field and is visible only while Instruction is the selected target.
- The popup contains an accessible **Valider les instructions** button that explicitly invokes the existing content-reformat action with the current context, instructions, and note content.
- Before invoking the model, pending context and instruction edits are acknowledged; if either cannot be saved, no reformat request is sent. The popup button and toolbar action share the same in-progress state and reject duplicate submissions.
- Selecting Contenu or Contexte, or toggling Instruction off, closes the popup and clears its text after any pending instruction segment is routed.
- The popup must not block selection of another mode.

## Editing and formatting

- Context and note content remain editable in all three dictation targets, including while capture is starting or listening.
- Instruction text is editable only while its popup is visible.
- Formatting buttons retain the existing command behavior for heading, bold, italic, paragraph, and bullet list.
- Each formatting button has a recognizable canonical-style icon, a stable accessible name, a tooltip, and visible keyboard focus.
- Existing Markdown rendering, unsupported-block preservation, context persistence, and note save safeguards remain unchanged.

## Explicit LLM action and privacy

- **Reformuler le contenu** remains a separate, explicit action available from the toolbar and the Instruction popup. Mode selection, popup opening/closing, capture startup, dictation, and manual editing do not request LLM work.
- The explicit action uses the latest acknowledged context and instructions with the current note content; failure to acknowledge either field prevents the request.
- Help remains available for disclosure that voice audio is sent to the configured transcription provider and that provider cost/retention are unknown.
- The status bar visibly indicates when audio capture is active and which target is receiving dictation.
- Existing configured-provider, key, and OS lifecycle checks remain in force; no provider fallback is permitted.

## State and accessibility contract

| UI condition | Required visible behavior |
|---|---|
| Application startup | No mode selected; microphone stopped. |
| Mode selected | Selected target is shown; capture shows starting, listening, or an actionable failure state. |
| Active mode clicked again | Capture stops; all mode buttons are deselected. |
| Target switch | Selected label and target status update; the prior capture ends and the new capture starts. |
| Instruction target | Borderless instructions popup is visible and editable. |
| Any other target | Instructions popup is hidden and its text is cleared after pending routing. |
| Reformulation in progress | Both explicit reformulation buttons are disabled and the in-progress status is announced. |
| Native capture starting | Status says starting; it does not claim active listening. |
| Native capture failure | Status/message communicates failure; no automatic retry loop is implied. |
| Note/application close | Capture ends; no active-listening status remains. |
| Reformulation not requested | No LLM action or request occurs. |
