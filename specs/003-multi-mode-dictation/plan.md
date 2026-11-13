# Implementation Plan: Dictée multimode et reformulation de notes

**Branch**: `lmirguet-capgemini-vocalnotes-version-2` | **Date**: 2026-11-13 | **Spec**: [spec.md](./spec.md)

**Input**: Functional specification in `specs/003-multi-mode-dictation/spec.md`.

## Summary

Separate the destination of dictated text (content, context, or instructions)
from the existing capture lifecycle, route each speech segment using the
destination captured when speech began, and add an explicit, content-only
reformat operation. Reuse the Tauri/ProseMirror/Markdown/OpenAI architecture;
persist context with the note, keep instructions ephemeral, and apply model
output only when the note and request inputs are still current.

## Technical Context

**Language/Version**: TypeScript 5.9.3 on Node.js 22.23.x; Rust 2024, rustc 1.99.1.

**Primary Dependencies**: Tauri 2.12, ProseMirror (`prosemirror-markdown`),
Serde, `pulldown-cmark`, existing OpenAI Responses transport. No new dependency
is planned.

**Storage**: Local Markdown note files with the existing versioned
`vocalnotes:intention:v1` HTML-comment metadata; context remains in the existing
metadata field. Mode selection and instructions are transient.

**Testing**: Vitest (`npm test`), TypeScript (`npm run typecheck` / `npm run
build`), Rust unit/integration tests (`cargo test --manifest-path
src-tauri/Cargo.toml`), and `cargo fmt --manifest-path src-tauri/Cargo.toml
-- --check`.

**Target Platform**: Existing Tauri desktop targets and native capture
adapters; preserve current Windows and Linux behavior. This feature adds no
platform.

**Project Type**: Desktop application with TypeScript UI and Rust/Tauri domain,
storage, and provider layers.

**Performance Goals**: No new latency SLA. Keep the existing 45-second
provider timeout, 16 KiB LLM request-context bound, 64 KiB response bound, and
1 MiB note-source bound. Reject oversized requests/results visibly; never
truncate note or prompt content.

**Constraints**:
- The native Rust domain remains authoritative for note content, revision,
  persistence, and stale-result checks.
- Dictation mode must not replace the existing capture state (`manual`,
  `voice`, `paused`); both concepts are represented independently.
- STT may continue during explicitly started voice capture. No LLM
  interpretation, interruption classification, or reformulation runs
  automatically; only the explicit reformat command may call the LLM.
- A reformat operation sends the current note content and only nonblank
  context/instructions. Its response is Markdown content only.
- Context is note data and persists locally. Instructions are transient and
  are cleared on leaving Instructions mode, note replacement, or close.
- If an instruction-targeted speech segment is pending when the user requests
  leaving Instructions mode, defer that switch until the segment is finalized
  and routed; then clear instructions as the mode changes.
- Preserve Markdown headings, bold, and italics. Tables and images are not new
  authoring features in this scope.

**Scale/Scope**: One active local note and one desktop user, following existing
session and file constraints. No collaboration, synchronization, or
multi-document redesign.

## Constitution Check

*Gate: evaluated before research and re-evaluated after design.*

| Principle | Result | Design impact |
|-----------|--------|---------------|
| I. Fidelity to speech | Pass | Dictation remains text input; reformulation is bounded to faithful restyling unless explicit instructions authorize content changes. |
| II. Two editing modes | **User-authorized deviation** | The v2 spec replaces the old voice/manual model with three dictation destinations and manual editing in each. The constitution must be aligned before implementation begins. |
| III. Note ownership and integrity | Pass | Keep local Markdown, metadata decoding protections, atomic save behavior, revision checks, and fail-closed handling. |
| IV. Privacy and security | Pass with clarified boundary | Persist only note context as part of the note. Keep instructions transient; send user fields to the configured LLM only after explicit reformat action, while existing voice consent still governs STT/audio transmission. |
| V. Simplicity and technical independence | Pass | Reuse current Tauri commands, Markdown editor, note metadata, and OpenAI transport; add only the typed operation and response contract required by v2. |

**Pre-research gate**: Pass with the explicitly user-authorized Principle II
deviation recorded below. No unresolved specification questions remain.

**Post-design gate**: Pass with the same deviation. The design adds no external
dependency, does not broaden data persistence, and preserves local note
integrity. Align Principle II with the approved v2 interaction model before
implementation; do not treat the old two-mode wording as a product constraint.

## Project Structure

### Documentation (this feature)

```text
specs/003-multi-mode-dictation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    ├── ui-ipc.md
    └── openai-reformat.md
```

### Source Code (repository root)

```text
src/
└── ui/
    ├── DesktopShell.ts             # UI coordination, snapshot application, reformat lifecycle
    ├── NoteView.ts                  # Existing Markdown/ProseMirror editor and serialization
    ├── Toolbar.ts                  # Dictation-target selector and explicit reformat action
    ├── contracts.ts                # Snapshot and typed IPC data
    ├── DesktopShell.test.ts
    ├── Prototype.editor.test.ts
    └── Prototype.test.ts

src-tauri/src/
├── app/
│   ├── commands.rs                 # Typed mode/context/instruction and reformat commands
│   ├── drafting.rs                 # Explicit reformat use case; no automatic LLM drafting
│   └── ipc_commands.rs             # Registered command surface
├── domain/
│   ├── note.rs                     # Note/context mutations and revision authority
│   ├── intention.rs                # Compatibility mapping for existing metadata
│   └── session.rs                  # Existing session tickets and stale-work barriers
├── ports/
│   └── interpreter.rs              # Content-only reformat contract
├── adapters/
│   └── openai_interpreter.rs       # Strict bounded Responses request and decode
├── platform/
│   ├── controller.rs               # Speech-start target capture and reformat orchestration
│   └── probe.rs                    # Snapshot/domain façade
└── storage/
    └── note_document.rs            # Existing versioned Markdown metadata format

tests/
├── drafting.rs
├── openai_interpreter.rs
├── prototype_pipeline.rs
└── storage_integrity.rs
```

**Structure Decision**: Extend the existing single Tauri desktop application.
Keep IPC at the current app command boundary, business invariants in the Rust
domain, provider formatting in the OpenAI adapter, and UI/editor behavior in
`src/ui`. Do not add a second application layer or a new storage format.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| Principle II still describes two editing modes | The approved v2 workflow explicitly replaces that interaction model. | Retaining it would contradict the three-mode specification and manual editing available in every mode; align the constitution before implementation instead. |
