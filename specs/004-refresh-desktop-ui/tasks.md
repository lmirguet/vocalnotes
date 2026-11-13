# Tasks: Refonte de l'interface desktop Vocalnotes

**Input**: Design documents from `specs/004-refresh-desktop-ui/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/ui-contract.md`, `quickstart.md`

**Tests**: Focused UI and native regression tests are included because the specification defines independent acceptance tests and measurable outcomes for all three user stories. Automated UI tests must mock Tauri actions and must not access the microphone or external providers.

**Organization**: Tasks are grouped by user story. Every task has an exact repository path; story tests precede implementation.

## Phase 1: Setup

**Purpose**: Reuse the existing project setup; no package or framework initialization is needed.

No dependency or project scaffolding changes are planned. Existing TypeScript, Tauri, Rust, ProseMirror, and test infrastructure will be reused.

---

## Phase 2: Foundational

**Purpose**: Resolve the documented governance prerequisite before changing shared capture behavior.

- [X] T001 Amend Principle II in `.specify\memory\constitution.md` to make the selected dictation mode own capture start/stop while preserving speech-start target routing, manual editing in every mode, and explicit LLM calls.

**Checkpoint**: The constitutional decision is recorded before any implementation task begins.

---

## Phase 3: User Story 1 - Rédiger dans une fenêtre desktop claire (Priority: P1) 🎯 MVP

**Goal**: Deliver the compact Windows editor shell, keep context above the dominant Markdown editor, and allow direct editing while capture is active.

**Independent Test**: Open a note, verify the compact shell and editor hierarchy, then edit context and note content in each dictation target; valid edits are accepted without overwriting newer revisions.

### Tests for User Story 1

- [X] T002 [P] [US1] Add failing UI tests for separate toolbars, context-above-editor layout, status placement, and editing fields in all targets in `src\ui\DesktopShell.test.ts`.
- [X] T003 [P] [US1] Add a failing domain regression test that accepts a valid manual content proposal during voice capture and rejects a stale revision in `src-tauri\src\domain\note.rs`.

### Implementation for User Story 1

- [X] T004 [US1] Allow valid `NoteCore::manual` proposals during capture without weakening note identity, manual-period, revision, or patch validation in `src-tauri\src\domain\note.rs`.
- [X] T005 [P] [US1] Restructure `DesktopShell` below the native title bar, separate the mode/reformat and formatting toolbars, and keep content/context editable in every target in `src\ui\DesktopShell.ts` and `src\ui\Toolbar.ts`.
- [X] T006 [US1] Style the compact desktop shell, separate toolbars, context panel, visible separator, dominant editor, status bar, and keyboard focus states in `src\style.css`.

**Checkpoint**: The note remains editable in every target and the core rejects stale or invalid edits.

---

## Phase 4: User Story 2 - Dicter dans le mode choisi sans commande vocale supplémentaire (Priority: P1)

**Goal**: Selecting a target starts its capture lifecycle and ends the previous one, with visible starting/listening/failure state and no separate voice controls.

**Independent Test**: With injected devices and mocked UI actions, verify startup with no selected mode and no capture, user selection of Contenu, same-mode stop/deselect, different-mode transition, pending Instruction routing on both transitions, and capture shutdown on note/application close; confirm the target status is truthful and no in-editor consent/start/resume/pause/stop controls exist.

### Tests for User Story 2

- [X] T007 [P] [US2] Add failing controller tests for initial capture, stop/drain/start on target changes, same-mode stop, pending Instruction segments, native startup failure, and close cleanup using injected devices in `src-tauri\src\platform\controller.rs`.
- [X] T008 [P] [US2] Add failing UI tests for unselected startup, same-mode stop/deselect, different-mode transitions, truthful capture status, no voice controls, and no LLM action from mode changes in `src\ui\DesktopShell.test.ts`.

### Implementation for User Story 2

- [X] T009 [US2] Coordinate pending dictation-target changes with capture termination so speech segments retain their start-time target and Instruction text clears only after pending routing in `src-tauri\src\domain\note.rs`.
- [X] T010 [US2] Route `SetDictationMode` through the controller to toggle off the active target or stop/drain the old capture and start a different target, retaining configured-provider and native lifecycle checks, in `src-tauri\src\app\commands.rs` and `src-tauri\src\platform\controller.rs`.
- [X] T011 [P] [US2] Remove the consent checkbox and graphical start/resume/pause/stop actions while retaining the three mode controls and explicit reformat action in `src\ui\Toolbar.ts`.
- [X] T012 [P] [US2] Convert session status to a read-only indicator of actual capture readiness/activity and active target, without a pause control, in `src\ui\SessionStatus.ts`.
- [X] T013 [US2] Keep capture stopped with no mode selected on application entry; start capture only after an explicit mode-button selection, and show startup/failure state plus audio disclosure in `src\ui\DesktopShell.ts`.

**Checkpoint**: The mode buttons control the capture lifecycle, and status never claims listening before native readiness.

---

## Phase 5: User Story 3 - Consulter les instructions uniquement au besoin (Priority: P1)

**Goal**: Show Instruction as a temporary borderless popup and provide recognizable, accessible mode and formatting icons.

**Independent Test**: Open Instruction, edit its text, switch away and verify the popup closes and pending text is cleared after routing; verify mode/style icons have accessible names and tooltips, and no LLM action occurs until Reformuler is selected.

### Tests for User Story 3

- [X] T014 [P] [US3] Add failing UI tests for popup visibility, deferred clearing on mode exit, accessible mode/style icon names and tooltips, and explicit-only reformulation in `src\ui\DesktopShell.test.ts`.

### Implementation for User Story 3

- [X] T015 [US3] Move the editable Instruction field into a mode-anchored popup and connect its lifecycle to the existing deferred-clear state in `src\ui\DesktopShell.ts` and `src\ui\Toolbar.ts`.
- [X] T016 [P] [US3] Add local SVG/glyph icons with visible labels, accessible names, and tooltips for Contenu, Contexte, Instruction, heading, bold, italic, paragraph, and bullet list in `src\ui\Toolbar.ts`.
- [X] T017 [P] [US3] Style the borderless Instruction popup, `.mode-icon`/`.format-icon` sizing, selected state, hover, and keyboard focus in `src\style.css`.

**Checkpoint**: Instructions are visible only in their mode, cleared on exit, and all mode/style actions remain understandable without relying on icons alone.

---

## Phase 6: Polish & Cross-Cutting Validation

**Purpose**: Verify the integrated UI, capture lifecycle, and project health without requiring a live microphone or provider call.

- [X] T018 Run the automated scenarios in `specs\004-refresh-desktop-ui\quickstart.md`, then run `npm test`, `npm run typecheck`, `npm run build`, and `cargo test --manifest-path src-tauri\Cargo.toml`; record any updated validation commands or outcomes in `specs\004-refresh-desktop-ui\quickstart.md`.
- [X] T019 [US3] Add mocked UI tests for the Instruction popup validation action, current context/instructions, failed field acknowledgement, and duplicate-submit prevention in `src\ui\DesktopShell.test.ts`.
- [X] T020 [US3] Add the popup validation button, synchronize its busy state with the toolbar, and document the explicit action contract in `src\ui\DesktopShell.ts`, `src\ui\Toolbar.ts`, `src\style.css`, and `specs\004-refresh-desktop-ui\`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependency changes or project initialization required.
- **Foundational (Phase 2)**: T001 must complete before any behavior implementation because Principle II currently describes capture and target as independent.
- **User Stories**: All depend on T001. US2 follows US1 because both modify the shared shell and mode controls; US3 follows US1/US2 because it integrates into the same toolbar and editor state.
- **Polish (Phase 6)**: T018 depends on completion of all three user stories.

### User Story Dependencies

- **US1 (P1)**: Starts after T001; independently delivers the compact shell and concurrent editing protections.
- **US2 (P1)**: Starts after US1; coordinates mode selection with the native capture session and removes obsolete voice controls.
- **US3 (P1)**: Starts after US2; adds the popup and icons to the final mode toolbar and verifies explicit-only LLM behavior.

### Within Each User Story

- Add and run failing regression tests before the corresponding implementation.
- US1: T002 and T003 are independent; after they are written, T004 and T005 can proceed in parallel; T006 follows the shell markup in T005.
- US2: T007 and T008 are independent; T009 precedes T010 for native transition sequencing; T011 and T012 can proceed in parallel; T013 integrates the completed native and toolbar behavior.
- US3: T014 precedes UI implementation; T015 establishes popup markup and lifecycle; T016 and T017 can proceed in parallel using the icon class names in the UI contract.

## Parallel Opportunities

- **US1**: Run T002 and T003 in parallel. After both test tasks, T004 (`note.rs`) and T005 (`DesktopShell.ts`) are separate-file implementation work; T006 follows T005.
- **US2**: Run T007 and T008 in parallel. Afterward, T011 (`Toolbar.ts`) and T012 (`SessionStatus.ts`) can run in parallel with T009/T010 native work; T013 integrates the results.
- **US3**: After T015 creates the popup and class hooks, T016 (`Toolbar.ts`) and T017 (`style.css`) can run in parallel.

## Parallel Examples

### User Story 1

```text
Parallel: T002 UI regression tests in src\ui\DesktopShell.test.ts
Parallel: T003 native edit-integrity test in src-tauri\src\domain\note.rs
Then parallel: T004 native acceptance and T005 shell/editor behavior
Then: T006 CSS layout after the shell class structure is established
```

### User Story 2

```text
Parallel: T007 controller lifecycle tests in src-tauri\src\platform\controller.rs
Parallel: T008 UI lifecycle tests in src\ui\DesktopShell.test.ts
Then: T009 pending-target handling before T010 controller/command orchestration
Parallel with native work: T011 Toolbar.ts controls and T012 SessionStatus.ts read-only status
Then: T013 DesktopShell.ts startup and transition integration
```

### User Story 3

```text
First: T014 popup/accessibility/reformat regression tests in src\ui\DesktopShell.test.ts
Then: T015 popup wiring in DesktopShell.ts and Toolbar.ts
Parallel: T016 local icons in Toolbar.ts
Parallel: T017 popup and icon styling in style.css
```

## Implementation Strategy

### MVP First (User Story 1)

1. Complete T001 so the constitutional rule matches the requested capture lifecycle.
2. Complete US1 to deliver the compact desktop editor and safe manual editing during capture.
3. Validate US1 independently with mocked capture state and domain tests.
4. Treat US1 as an intermediate MVP only; do not consider the requested redesign complete until US2 and US3 remove the old voice controls and deliver automatic mode behavior.

### Incremental Delivery

1. Complete US1 and validate the desktop editor and edit-integrity protections.
2. Complete US2 and validate automatic start/stop, pending-segment routing, status, and shutdown.
3. Complete US3 and validate the Instruction popup, accessible icons, and explicit reformat action.
4. Complete T018 and confirm the quickstart scenarios and build/test suites.

## Notes

- `[P]` tasks touch distinct files and can be worked on in parallel only after their listed prerequisites.
- All tasks use the checklist format `- [ ] T### [P?] [US#?] Description with file path`.
- No new dependency, persisted data field, table/image support, or live-provider test is introduced.
