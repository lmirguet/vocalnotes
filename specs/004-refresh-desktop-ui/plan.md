# Implementation Plan: Refonte de l'interface desktop Vocalnotes

**Branch**: `lmirguet-capgemini-vocalnotes-version-2` | **Feature**: `004-refresh-desktop-ui` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/004-refresh-desktop-ui/spec.md`

## Summary

Align the existing Windows desktop editor with the approved compact mockup: arrange modes, explicit reformatting, formatting, context, note editor, and status in a dense desktop shell below the native Windows title bar; expose Instruction as a temporary borderless popup with an explicit validation button; and let the three mode buttons control capture. Start with no mode selected and microphone capture stopped; an inactive mode starts capture, clicking the active mode stops and deselects, and clicking another mode changes target. Keep current Markdown/note persistence and explicit LLM reformat behavior. Reuse the TypeScript DOM, ProseMirror, Tauri, and Rust controller architecture. The voice transition must preserve the existing mode-at-speech-start routing and pending-instruction drain; manual editing must remain available while capture is active.

## Technical Context

**Language/Version**: TypeScript 5.9.3 for the UI; Rust 1.99.1, edition 2024, for the native core.

**Primary Dependencies**: Tauri 2.12; Vite 7.3; ProseMirror 1.25; Vitest 4.1. Use existing DOM helpers and local SVG/CSS; add no UI or icon package.

**Storage**: Existing local Markdown note storage with context persistence. Instructions remain temporary. No new persisted entity or format.

**Testing**: `npm test`, `npm run typecheck`, `npm run build`, and `cargo test --manifest-path src-tauri\Cargo.toml`; UI tests use mocked Tauri `Invoke` for the popup validation flow, save failures, and duplicate prevention, while Rust controller tests use injected session devices.

**Target Platform**: Windows desktop application built with Tauri; no browser/mobile redesign.

**Project Type**: Desktop app with a TypeScript DOM renderer and Rust native lifecycle/controller.

**Performance Goals**: No new throughput target. Mode selection updates its visible target immediately; microphone startup/failure is reported through the existing status state rather than shown as ready prematurely.

**Constraints**: Keep voice-mode authorization implicit as the user specified, but retain configured-model/account checks, native OS lifecycle guards, active-microphone status, and a clear Help disclosure that audio is sent to the configured transcription provider. Do not trigger LLM reformatting except on its explicit action. Preserve pending-segment routing, Markdown integrity, and local note safety. No tables/images or new editor capabilities.

**Scale/Scope**: One desktop note window, three dictation targets, one temporary instruction field, the existing formatting actions, and the existing status/menus. Redesign the shell and coordinate UI mode changes with the native capture lifecycle; do not redesign preferences, note formats, or providers.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Check |
|---|---|---|
| I. Fidelity to speech | PASS | Dictation remains routed to its selected target; LLM changes remain explicit and content-preserving unless instructed otherwise. |
| II. Multimode dictation and explicit control | PASS — USER-DIRECTED CHANGE | The current constitution says capture state is independent of the selected destination. The user's explicit request supersedes that behavior: selecting a mode owns capture start/stop. Amend this principle before implementation while preserving target-at-speech-start routing and explicit LLM calls. |
| III. Note ownership and integrity | PASS | Keep local Markdown storage, context persistence, revision checks, and safe close/save behavior. Manual edits during capture must use the existing integrity/revision protections. |
| IV. Privacy and security | PASS WITH DISCLOSURE | The user requested removal of the in-editor consent/start/stop controls. Keep the configured provider and native OS lifecycle guards, visibly expose active listening/target, and retain clear Help disclosure about audio transmission, cost, and retention. |
| V. Simplicity and technical independence | PASS | Reuse the existing DOM/Tauri architecture, commands, and tests; no new package or provider abstraction. |

**Gate decision**: PASS for planning. No unresolved technical unknowns remain. The only intentional change to a constitutional invariant is directly specified by the user; amending Principle II is a required first implementation task, before code changes. Removing redundant controls does not authorize removal of native lifecycle checks, provider configuration, or audio-transmission disclosure.

### Post-design re-check

The design retains the existing separate target and capture-state data, local note integrity, configured provider checks, OS lifecycle guards, and explicit LLM action. It changes only the user-directed relationship between target selection and capture lifecycle; the planned Principle II amendment records that exception. No additional dependency or architecture layer is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/004-refresh-desktop-ui/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ui-contract.md
└── tasks.md                 # Created by /speckit-tasks
```

### Source Code (repository root)

```text
src/
├── style.css
└── ui/
    ├── DesktopShell.ts
    ├── Toolbar.ts
    ├── SessionStatus.ts
    ├── contracts.ts
    └── DesktopShell.test.ts

src-tauri/
├── src/
│   ├── app/commands.rs
│   ├── audio/capture.rs
│   ├── domain/note.rs
│   └── platform/controller.rs
└── tests/

.specify/
└── memory/constitution.md
```

**Structure Decision**: Keep the current single desktop application structure and Tauri's native Windows title bar. `DesktopShell.ts`, `Toolbar.ts`, `SessionStatus.ts`, and `style.css` own the in-window shell; the native mode/capture transition stays in the existing command, domain, and controller paths. Extend `DesktopShell.test.ts` and Rust tests rather than introducing a new frontend subsystem.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| Amend Constitution Principle II from capture/destination independence to mode-owned automatic capture. | The user explicitly chose that activating a dictation mode starts capture and leaving it ends capture, with no separate voice controls. | Keeping capture independent would contradict the approved behavior and would either leave misleading controls/status or fail to start dictation from the mode buttons. |
