# Research — Dictée multimode et reformulation de notes

**Date**: 2026-11-13

**Status**: Design decisions resolved from the feature specification and current repository architecture.

## 1. Keep dictation destination separate from capture state

**Decision**: Add a typed dictation destination (`content`, `context`,
`instructions`) independent of the existing capture lifecycle (`manual`,
`voice`, `paused`). Capture the destination when a speech segment starts and
carry that immutable target with the finalized segment.

**Rationale**: `Snapshot.mode` and the controller currently describe the voice
session lifecycle, while `PassageGate::Start` identifies the beginning of a
speech segment and `Finalized` carries it through transcription. Reusing the
existing `mode` would conflate two unrelated state machines and could route a
late transcript to the wrong field.

**Alternatives considered**: Reinterpret `mode` as the three dictation modes;
rejected because capture controls and lifecycle logic already depend on its
manual/voice/paused meaning. Routing by the mode at transcription completion
was rejected because the specification requires the mode at speech onset.

**Instruction exit edge case**: When the user requests leaving Instructions
while an instruction-targeted segment is in flight, defer the mode change
until all such pending segments are finalized and routed, then clear the
instruction text. This is the user's clarification recorded in `spec.md`.

## 2. Reuse the note metadata and Markdown editor

**Decision**: Keep local Markdown as the note source of truth and retain the
current versioned `vocalnotes:intention:v1` metadata envelope. Expose its
existing `context` value as note context, preserving hidden legacy metadata
fields when reading and writing older notes. Keep the current ProseMirror
parser/serializer and toolbar formatting commands.

**Rationale**: `storage/note_document.rs` already stores
`DraftingIntention.context` with a note, rejects malformed or unsupported
metadata instead of stripping it, and preserves Markdown source. `NoteView.ts`
already uses `prosemirror-markdown` for the supported editor schema, while
`Toolbar.ts` contains the formatting controls to extend. This directly supports
the requested saved context and Markdown styling without a migration or new
editor dependency.

**Alternatives considered**: Introduce a new sidecar/context file or replace
the note metadata format; rejected because it adds migration and consistency
risks without a product need. Add tables/images; excluded by the specification.

## 3. Use a separate, explicit reformat contract

**Decision**: Add a dedicated interpreter operation for a single Markdown
string input/output. Invoke it only from the user's reformat action. The
backend assembles the current content, nonblank context, and nonblank
instructions from its authoritative state; the frontend does not submit
replacement note content as an LLM command.

**Rationale**: The current `Interpreter::interpret_final` accepts a speech
passage and returns a structured `Intervention` whose actions include note
edits, drafting-intention updates, and terminal session commands.
`interpret_interruption` is also an automatic LLM call on provisional speech.
Neither contract can enforce v2's content-only update and explicit-call
requirements. `OpenAIInterpreter::request` already centralizes the Responses
transport, timeout, response bounds, and strict JSON-schema format; it can
support a separate schema without weakening the current validation.

**Alternatives considered**: Reuse `interpret_final`; rejected because it
permits unrelated commands and does not return a replacement document.
Keep automatic interruption interpretation; rejected because v2 says the LLM
is called only by explicit user action. Do not add a keyword-based replacement
for those commands without a separate product requirement.

## 4. Preserve data on failures and stale responses

**Decision**: Capture note identity, revision, and a generation for mutable
reformat inputs when a request starts. Apply a response only if the note and
all relevant inputs remain unchanged. Reject empty, oversized, malformed,
stale, or unusable output without modifying content. Preserve opaque Markdown
blocks by validating that they are unchanged before commit.

**Rationale**: The note domain already uses note IDs, revisions, generations,
and tickets to prevent stale asynchronous work from affecting newer edits.
`note_document.rs` and storage tests establish fail-closed behavior for
unsupported or corrupt metadata. The 1 MiB note source, 16 KiB LLM request
context, 64 KiB response, and 45-second timeout are established repository
bounds. Reformatting must not bypass these protections or truncate user data.

**Alternatives considered**: Apply any response when it arrives; rejected
because it can overwrite new manual edits or context/instruction changes.
Truncate inputs or output to fit provider bounds; rejected because it silently
changes user content. Modify unknown Markdown structures on best effort;
rejected because that can corrupt content outside the supported editor scope.

## 5. Data and privacy boundary

**Decision**: Persist only context with its note. Keep instructions and the
selected dictation destination in memory. Do not log field contents. A reformat
click sends the current content and nonblank context/instructions to the
configured LLM; the existing voice consent remains responsible for audio and
STT transmission during voice capture.

**Rationale**: The clarified specification requires context restoration and
instruction clearing. The constitution limits durable user content to the
note and requires external sends to be necessary and transparent. The existing
OpenAI adapter uses `store:false`, a bounded request, and explicit transport
error handling; this does not imply provider-side zero retention.

**Alternatives considered**: Persist instructions or mode with the note;
rejected because they are transient according to the specification. Send
fields during edits or mode selection; rejected because the LLM call must be
explicit.

## 6. Repository evidence

- `src-tauri/src/platform/controller.rs`: VAD segment start/final pipeline,
  existing automatic final and provisional LLM paths, and session tickets.
- `src-tauri/src/platform/pipeline.rs`: `PassageStep::Start` timing used to
  associate destination with speech onset.
- `src-tauri/src/app/drafting.rs` and `src-tauri/src/ports/interpreter.rs`:
  current automatic interpretation and structured action contract.
- `src-tauri/src/adapters/openai_interpreter.rs`: bounded Responses transport,
  strict schema, `store:false`, 45-second timeout, 16 KiB request and 64 KiB
  response limits.
- `src-tauri/src/domain/note.rs`, `src-tauri/src/domain/intention.rs`, and
  `src-tauri/src/storage/note_document.rs`: note authority, revision gates,
  context metadata, and Markdown-file encoding.
- `src/ui/NoteView.ts`, `src/ui/Toolbar.ts`, `src/ui/DesktopShell.ts`, and
  `src/ui/contracts.ts`: current Markdown editor, toolbar, IPC orchestration,
  and snapshot shape.
- `specs/002-remplacer-services-openai/contracts/ui-ipc.md` and
  `specs/002-remplacer-services-openai/data-model.md`: established repository
  conventions for contracts and model documentation.
