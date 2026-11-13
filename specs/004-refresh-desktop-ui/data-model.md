# Data Model: Refonte de l'interface desktop Vocalnotes

This feature adds no persisted note fields. It clarifies how existing editor data, the optional selected dictation target, and native capture state interact.

## Entities

### Note

- **Identity**: Existing note identifier and local Markdown destination.
- **Content**: Markdown source and structured blocks; edited through ProseMirror.
- **Context**: Persistent text associated with the note.
- **Instructions**: Temporary text associated with the current editing session; cleared when leaving Instruction after pending segments have been routed.
- **Revision state**: Existing revision, manual period, input generation, and transaction identities used to reject stale edits or results.

### Dictation target

- **Values**: `content`, `context`, `instructions`.
- **Purpose**: Selects the destination for each spoken segment.
- **Initial selection**: None. The application remains in manual editing until the user selects a target.
- **Toggle behavior**: Selecting the active target stops capture and clears the active selection; selecting a different target transitions to it.
- **Invariant**: A segment remains bound to the target active when speech began, even if a mode transition is requested before transcription completes.

### Capture session

- **Runtime state**: `stopped`, `starting`, `listening`, or a reported failure.
- **Configuration**: Existing selected transcription model/provider and native session lifecycle.
- **Lifetime**: Started for the active dictation target; stopped when that target ends, a different target is selected, or the note/application closes.
- **Persistence**: Audio and transcript processing remain temporary; no new session data is written to the note.

### Editor status

- **Visible values**: Active target, capture readiness/activity, existing STT/LLM counters and provider status where available.
- **Purpose**: Keep automatic capture observable without dedicated start/stop/permission controls.

## Relationships and state transitions

1. Application startup leaves the dictation target unselected and capture stopped.
2. Selecting a target requests its capture session.
3. Selecting the active target drains and stops capture, clears temporary instructions if leaving `instructions`, and leaves no target selected.
4. Selecting a different target ends the old capture session and prevents new audio from being assigned to it.
5. Already-started speech segments retain their original target. On leaving `instructions`, pending instruction segments are finalized and applied before the instruction text is cleared.
6. After the old session has been stopped and pending target-specific work is settled, the new target becomes active and capture starts for it.
7. Manual edits remain valid while capture is active and update the existing revision/input-generation state. A stale voice or LLM result must not overwrite newer edits.
8. Closing the note or application stops capture and leaves the visible status non-active.

## Validation rules

- Context is edited and saved with the note using existing validation and storage rules.
- Instructions are editable only while the Instruction target is active and remain bounded by existing limits.
- Content edits, formatting operations, voice updates, and LLM updates continue to use the current integrity checks; unsupported Markdown blocks remain protected.
- A capture state is never displayed as `listening` until the native capture reports readiness.
- The LLM remains triggered only by the explicit reformulation action.
