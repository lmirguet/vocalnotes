---
description: "Implementation tasks for Vocalnotes v2 multi-mode dictation and explicit reformatting"
---

# Tasks: Dictée multimode et reformulation de notes

**Input**: Design documents from `specs/003-multi-mode-dictation/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/`, and `quickstart.md`

**Tests**: The specification includes mandatory user-story test scenarios and
independent test criteria. Focused UI, domain, storage, and provider-contract
tests are included before their implementation tasks. Use fake transports;
automated tests must not call OpenAI.

**Organization**: Tasks are grouped by the three P1 user stories. Each story
has an independent test criterion; US3 depends on the note fields delivered by
US1 and US2.

## Phase 1: Setup

**Purpose**: Align project governance with the explicitly approved v2
interaction model before implementation.

- [X] T001 Update Principle II to the approved three dictation destinations, manual editing in every mode, and explicit-only LLM behavior in `.specify/memory/constitution.md`, preserving the existing fidelity, local-note integrity, and privacy principles.

---

## Phase 2: Foundational

**Purpose**: Define shared state and IPC types required by all user stories.

- [X] T002 [P] Add a typed `DictationTarget` with exactly `content`, `context`, and `instructions`, separate from capture state `manual` / `voice` / `paused`, and add transient instruction and pending-transition state in `src-tauri/src/domain/note.rs` and `src-tauri/src/domain/mod.rs`.
- [X] T003 [P] Extend `Snapshot` and typed UI contracts with `dictationMode`, persistent note `context`, transient `instructions`, pending-mode-transition status, and reformat input generation in `src/ui/contracts.ts`.
- [X] T004 Add strict native actions for setting dictation mode and instructions to the existing `prototype_action` boundary in `src-tauri/src/app/commands.rs` (depends on T002 and T003; context mutation follows in US2).

**Checkpoint**: Shared model and IPC shapes are stable; user-story work can
begin.

---

## Phase 3: User Story 1 - Dicter dans le champ correspondant au mode actif (Priority: P1)

**Goal**: Route each finalized speech segment to the field selected when that
segment began, including the deferred exit from Instructions.

**Independent Test**: Select content, context, and instructions in turn and
dictate distinct phrases. Verify each final transcript changes only its
captured destination. If an instruction-targeted segment is pending when the
user requests another mode, verify the switch waits for finalization and
routing, then clears instructions.

### Tests for User Story 1

- [X] T005 [P] [US1] Add native pipeline tests for captured-target routing, delayed STT, and pending Instructions-mode exit in `src-tauri/tests/prototype_pipeline.rs`.
- [X] T006 [P] [US1] Add UI tests for the three selectable destinations, selected-mode indication, and deferred Instructions exit in `src/ui/Prototype.test.ts`.

### Implementation for User Story 1

- [X] T007 [US1] Capture `DictationTarget` when `PassageStep::Start` begins and route the finalized STT text only to that target in `src-tauri/src/platform/controller.rs`, `src-tauri/src/platform/pipeline.rs`, and `src-tauri/src/domain/note.rs`.
- [X] T008 [US1] Remove automatic final-passage and provisional-interruption LLM calls so finalized STT routes directly as text; update the obsolete path and its tests in `src-tauri/src/platform/controller.rs`, `src-tauri/src/app/drafting.rs`, `src-tauri/src/ports/interpreter.rs`, `src-tauri/src/adapters/openai_interpreter.rs`, `src-tauri/tests/openai_priority.rs`, and `src-tauri/tests/provider_contracts.rs`.
- [X] T009 [US1] Add the three-mode selector and accessible selected-state display to `src/ui/Toolbar.ts`, `src/ui/DesktopShell.ts`, and `src/style.css` without changing the existing capture controls.
- [X] T010 [US1] Implement the deferred mode transition: keep Instructions active until all already-pending instruction-targeted segments are finalized and routed, then clear the field and change modes in `src-tauri/src/domain/note.rs`, `src-tauri/src/platform/controller.rs`, and `src/ui/DesktopShell.ts`.

**Checkpoint**: Dictation routes correctly without an automatic LLM call;
mode changes and pending instruction transcripts obey the clarified sequence.

---

## Phase 4: User Story 2 - Saisir et organiser manuellement les informations (Priority: P1)

**Goal**: Keep context and content manually editable in every mode, expose
instructions only in Instructions mode, persist note context, and preserve
supported Markdown styling.

**Independent Test**: Edit context and content manually in each mode and edit
instructions only in Instructions mode. Save and reopen a note to verify its
context and Markdown styles return while instructions do not.

### Tests for User Story 2

- [X] T011 [P] [US2] Add UI editor tests for manual context/content/instruction edits, visibility by mode, and heading/bold/italic Markdown round-trips in `src/ui/Prototype.editor.test.ts` and `src/ui/markdown.test.ts`.
- [X] T012 [P] [US2] Add storage tests for context metadata round-trip, legacy metadata preservation, and instructions not being serialized in `src-tauri/tests/storage_integrity.rs`.

### Implementation for User Story 2

- [X] T013 [US2] Preserve the legacy `purpose`, `form`, and `tone` metadata while exposing and updating `context`; enforce the existing verbatim limit “4096 octets cumulés sur ses champs legacy et contexte” in `src-tauri/src/domain/intention.rs`, `src-tauri/src/domain/note.rs`, and `src-tauri/src/storage/note_document.rs`.
- [X] T014 [US2] Render the permanent context field above the larger Markdown content editor and the editable instructions field only in Instructions mode; keep all visible fields manually editable in `src/ui/DesktopShell.ts` and `src/style.css`.
- [X] T015 [US2] Preserve editable heading, bold, and italic Markdown through load/edit/serialize operations, keep unsupported opaque blocks unchanged, and do not add table or image authoring in `src/ui/NoteView.ts`.
- [X] T016 [US2] Persist context through the existing note metadata/save lifecycle and clear transient instructions on mode exit, note replacement, and close in `src-tauri/src/app/commands.rs`, `src-tauri/src/domain/note.rs`, `src-tauri/src/storage/note_document.rs`, and `src/ui/DesktopShell.ts`.

**Checkpoint**: Manual editing, note-context restoration, transient instructions,
and supported Markdown styles work independently of voice routing.

---

## Phase 5: User Story 3 - Reformater le contenu à la demande (Priority: P1)

**Goal**: Send current content plus nonblank context/instructions only on an
explicit toolbar action, accept a content-only result, and reject failures or
stale results without data loss.

**Independent Test**: Confirm edits, dictation, and mode changes make no LLM
request. Click Reformulate once and verify exactly one strict request with the
current fields. Verify valid output changes only content; simulate an error or
concurrent edit and verify the latest note remains intact with visible
feedback.

### Tests for User Story 3

- [X] T017 [P] [US3] Add use-case tests for empty content, request bounds, content-only application, and stale note/input revisions in `src-tauri/tests/drafting.rs`.
- [X] T018 [P] [US3] Add OpenAI adapter contract tests for the strict `content_markdown` response, omitted blank fields, size bounds, malformed output, refusal, and timeout in `src-tauri/tests/openai_interpreter.rs`.
- [X] T019 [P] [US3] Add UI tests proving that only the explicit Reformulate action invokes `prototype_reformat`, with duplicate-click, busy, error, and stale-result feedback in `src/ui/DesktopShell.test.ts`.

### Implementation for User Story 3

- [X] T020 [US3] Define a dedicated content-only reformat request/result and interpreter port, separate from speech interventions and session actions, in `src-tauri/src/app/drafting.rs` and `src-tauri/src/ports/interpreter.rs`.
- [X] T021 [US3] Implement the strict OpenAI Responses request and `content_markdown` schema in `src-tauri/src/adapters/openai_interpreter.rs`; include only nonblank context/instructions, use `store:false`, retain the 45-second timeout, reject serialized input over 16 KiB and response over 64 KiB, and never retry or truncate.
- [X] T022 [US3] Add native `prototype_reformat` orchestration that reads fields from authoritative state, refuses empty content, and commits only when note ID, revision, and input generation still match; reject changed opaque blocks and note output over 1 MiB in `src-tauri/src/platform/controller.rs`, `src-tauri/src/domain/note.rs`, `src-tauri/src/app/commands.rs`, `src-tauri/src/app/ipc_commands.rs`, and `src-tauri/src/lib.rs`.
- [X] T023 [US3] Add the explicit Reformulate toolbar action, one-operation busy state, and visible success/failure status in `src/ui/Toolbar.ts`, `src/ui/DesktopShell.ts`, and `src/ui/contracts.ts`; send no note fields through IPC and make no request on editing, dictation, mode changes, or saves.

**Checkpoint**: The configured LLM is called only by user action; valid,
current output replaces only note content, while invalid or stale output is
visible and has no effect.

---

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Verify the integrated stories against the repository checks and
the feature acceptance scenarios.

- [ ] T024 Run the automated and manual scenarios in `specs/003-multi-mode-dictation/quickstart.md`, verify all Story 1-3 acceptance criteria, and update that file only if implementation changes the validated commands or expected behavior.

---

## Dependencies and Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 aligns the constitution before implementation.
- **Foundational (Phase 2)**: T002 and T003 are independent after T001; T004
  depends on both and blocks all user stories.
- **User Stories (Phases 3-5)**: US1 and US2 depend on Phase 2. They may be
  developed as separate increments, but shared UI/domain integration should be
  sequenced to avoid conflicting edits. US3 depends on both US1 and US2 because
  it consumes their content, context, and instructions state.
- **Polish (Phase 6)**: T024 depends on all three user stories.

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2; no dependency on another story.
- **US2 (P1)**: Can start after Phase 2; its shared editor integration should
  follow or coordinate with US1's toolbar and shell changes.
- **US3 (P1)**: Starts after US1 and US2 provide the authoritative fields and
  UI state; it must not reintroduce automatic STT-triggered LLM calls.

### Parallel Opportunities

- **Setup/foundation**: T002 and T003 can run in parallel; T004 follows both.
- **US1**: T005 and T006 can run in parallel. After them, T007 (native routing)
  and T009 (toolbar selector) can proceed in parallel; T008 and T010 depend on
  the native routing/state work. T010 also follows T009 because it integrates
  the deferred native transition with its UI status.
- **US2**: T011 and T012 can run in parallel. T013 (native metadata) and T014
  (UI fields/layout) can proceed in parallel after the tests; T015 can proceed
  alongside T013 because it changes the editor schema, not metadata storage.
- **US3**: T017, T018, and T019 can run in parallel. T020 and T021 can proceed
  in parallel after the contract tests; T022 follows the native contract and
  T023 follows the command shape.
- **Across stories**: US1 and US2 may be split across owners after foundation,
  but shared `DesktopShell.ts`, `Toolbar.ts`, and `domain/note.rs` changes
  should not be edited concurrently without coordination.

### Parallel Execution Examples

#### User Story 1

```text
Parallel: T005 native pipeline tests
Parallel: T006 UI mode-selection tests
Then: T007 native target capture and routing
Parallel after T007: T008 remove automatic LLM paths; T009 mode selector UI
Then: T010 deferred Instructions-mode exit
```

#### User Story 2

```text
Parallel: T011 UI manual-edit/Markdown tests
Parallel: T012 Rust storage compatibility tests
Then in parallel: T013 metadata compatibility; T014 context/instructions layout
In parallel with T013: T015 Markdown preservation
Then: T016 persistence/lifecycle integration after T013-T015
```

#### User Story 3

```text
Parallel: T017 use-case tests
Parallel: T018 OpenAI contract tests
Parallel: T019 UI explicit-trigger tests
Then in parallel: T020 dedicated domain port; T021 strict OpenAI adapter
Then: T022 native stale-safe command; T023 explicit toolbar integration
```

## Implementation Strategy

### MVP First

Recommended MVP: **US1 only**, after Setup and Foundational. It demonstrates
separate dictation destinations, speech-onset routing, and the clarified
pending-instruction transition without automatic LLM calls. US2 adds complete
manual editing and persisted context; US3 adds explicit reformulation.

### Incremental Delivery

1. Complete T001-T004 and verify the shared types and command boundary.
2. Complete US1 (T005-T010) and validate its independent routing criteria.
3. Complete US2 (T011-T016) and validate manual editing, context restoration,
   transient instructions, and Markdown.
4. Complete US3 (T017-T023) and validate the explicit-only LLM contract and
   stale-result protection.
5. Run T024 against the full quickstart and success criteria.

## Notes

- Every task uses the required `- [ ] T### [P?] [US#?] ...` format and names
  its target file path.
- `[P]` is reserved for work in distinct files with no dependency on unfinished
  tasks.
- User-story tests are included because the feature specification explicitly
  requires independent test criteria and acceptance scenarios.
- No package or Cargo dependency is added.
