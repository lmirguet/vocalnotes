# Quickstart: Refonte de l'interface desktop Vocalnotes

## Prerequisites

- Windows desktop environment for visual smoke testing.
- Node.js `>=22.23.2 <23`, npm, Rust `1.99.1`, and the existing project dependencies.
- A configured transcription provider and Windows audio lifecycle support are required only for a live microphone smoke test.

## Automated validation

From the repository root:

```powershell
npm ci
npm test
npm run typecheck
npm run build
cargo test --manifest-path src-tauri\Cargo.toml
```

The UI tests must use a mocked Tauri `Invoke` function and must not open the microphone or contact the transcription/LLM provider.

## UI scenarios

1. Mount the shell with a synthetic snapshot. Confirm the compact desktop layout, separate formatting toolbar, context panel above the editor, and status bar.
2. Confirm no mode is selected on startup, the status reports the microphone stopped, and no mode-change/capture command is invoked. Click Contenu and confirm the mocked capture transition reports `starting` then `listening`; verify there are no consent/start/resume/pause/stop controls in the editor toolbar.
3. Click Contenu again and confirm capture stops and all three mode buttons are deselected. Then select Contexte and Instruction in turn; confirm each different-mode selection drains/stops the old capture and starts the new target.
4. While Instruction has a pending speech segment, click Instruction again. Confirm the segment is routed before its temporary text clears and the mode buttons become deselected.
5. Type in the context and Markdown editor while the capture state is listening. Confirm both edits are accepted and a later stale speech result cannot overwrite the newer note revision.
6. Select Instruction, edit its popup field, then switch away. Confirm the popup closes and the field clears after pending instruction routing.
7. Inspect heading, bold, italic, paragraph, and bullet-list actions for icon recognition, tooltip, accessible name, and keyboard focus.
8. Type, dictate, and switch modes without selecting Reformuler. Confirm there is no LLM action; select Reformuler or, in Instruction, click **Valider les instructions** and confirm the current context/instructions are acknowledged and only note content is updated. Verify popup opening/closing alone does not call the LLM, double submission is blocked, and an unacknowledged field prevents a request.
9. Close the note/application in the native lifecycle tests and confirm capture is stopped.

## Optional live smoke test

`npm run tauri -- dev` launches the actual desktop app with no mode selected and capture stopped. It is safe to verify the initial window without activating audio; do not click a mode button unless the operator intends to use the microphone and the configured transcription provider. Check that the initial status says no mode is selected and the microphone is stopped. Clicking the active mode again stops capture and deselects all modes. Do not use a live microphone or provider call as a prerequisite for the automated UI validation.

See [the UI contract](contracts/ui-contract.md) for user-visible states and [the data model](data-model.md) for capture and target transitions.

## Validation results

- `npm test`: passed, 77 tests across 12 files, including the popup validation flow, failed field acknowledgement, and duplicate-submit protection.
- `npm run typecheck` and `npm run build`: passed.
- `npm test -- --run src/ui/DesktopShell.test.ts`: passed, 17 UI tests using mocked Tauri actions; no microphone or live provider was used.
- `cargo test --manifest-path src-tauri\Cargo.toml platform::controller::tests`: passed, 10 controller tests using simulated devices; no live microphone or provider was used for this change.
- `rustfmt --edition 2024 --check src-tauri\src\platform\controller.rs`: passed. The repository-wide `cargo fmt --check` still reports a pre-existing extra blank line in `src-tauri/src/domain/note.rs`.
