# Research: Refonte de l'interface desktop Vocalnotes

## Decisions

### Reuse the existing desktop renderer

**Decision**: Implement the visual shell with the current TypeScript DOM helpers and `style.css`, retaining ProseMirror for the Markdown editor and Tauri for native actions.

**Rationale**: The repository already builds and tests this UI path. `DesktopShell.ts` composes the menu, toolbar, note fields, editor, status, and preferences; replacing the renderer or adding a component framework is unnecessary for a single-window layout change.

**Alternatives considered**: Replacing the frontend framework or adding a desktop component library was rejected because it adds dependencies and migration risk without enabling required behavior.

### Keep the native Windows title bar

**Decision**: Use the existing decorated Tauri window for the title bar and place the compact menu/toolbar/editor layout within its client area.

**Rationale**: `src-tauri/tauri.conf.json` already configures the app title, window dimensions, and minimum size, with native decorations enabled by default. A second HTML title bar and fake minimize/close buttons would duplicate OS controls and make the frontend less like a real Windows desktop app.

**Alternatives considered**: Recreating the title bar in the DOM was rejected because the current native window already provides it and the mockup is only a visual reference.

### Use local, accessible icons

**Decision**: Draw mode and formatting icons with local SVG or text glyphs; mark decorative SVG as hidden from assistive technology and give each control an accessible name and tooltip.

**Rationale**: `Toolbar.ts` already creates native buttons and has no icon package. Local icons satisfy the mockup without remote assets, package changes, or altered Markdown commands.

**Alternatives considered**: An icon package or remote icon font was rejected to avoid a new dependency and network asset requirements.

### Coordinate selected dictation mode with capture lifecycle

**Decision**: Keep the domain distinction between dictation target and capture runtime state, but coordinate them as one user-visible mode lifecycle: start with no target selected and capture stopped; selecting an inactive mode starts capture; clicking the active mode stops and deselects; switching modes stops/drains the old capture before applying and starting the next target. Preserve target-at-speech-start routing and delay leaving Instruction until pending instruction segments are routed and cleared.

**Rationale**: `Snapshot` already distinguishes `dictationMode` from `mode` and has `listeningState`; the UI currently sends `set_dictation_mode` separately from `start`/`pause`/`stop`. `Controller` owns the native session and injectable devices, while `NoteCore` already handles pending Instruction transitions. A single coordinated transition avoids duplicating capture state in the frontend.

**Alternatives considered**: Merely hiding the controls while leaving capture manual was rejected because the selected mode would not start dictation. Treating the target itself as the capture status was rejected because startup, native failure, and listening readiness are distinct runtime states.

### Preserve explicit privacy and lifecycle signals without voice controls

**Decision**: Remove the editor's consent/start/resume/pause/stop controls as requested, but retain selected provider/model checks, native OS lifecycle guards, a persistent listening/target status, and Help disclosure of audio transmission and unknown provider retention/cost. A user click on a mode button explicitly starts capture for that target; do not start capture on application launch, silently substitute a provider, or auto-trigger LLM work.

**Rationale**: The current native start command verifies configuration and OS lifecycle, and Help already describes the external audio flow. The status bar and Help menu preserve transparency without adding controls the user rejected.

**Alternatives considered**: Keeping a separate in-editor permission checkbox or start/stop buttons contradicts the mockup. Bypassing native lifecycle checks or hiding audio transmission was rejected as inconsistent with the repository's privacy and safety principles.

### Allow manual editing during capture with existing revision protections

**Decision**: Enable context, instruction, content, and formatting edits while a voice session is active, and keep the existing revision/generation checks for voice and LLM results.

**Rationale**: The specification requires editing in all three modes. Currently the ProseMirror editor is editable only in `manual` state and `NoteCore::manual` accepts changes only there. Allowing edits must therefore be represented in the native domain and covered by stale-result tests, not just by changing CSS.

**Alternatives considered**: Making the editor appear enabled while the core rejects edits was rejected as misleading. Buffering all manual edits until capture stops was rejected because it would not satisfy direct editing in every mode.

### Keep the data and provider model unchanged

**Decision**: Add no persisted fields or external provider contracts. Reuse note context/instructions, current status metrics, existing voice providers, and explicit reformulation.

**Rationale**: This is a shell and lifecycle change. Notes continue to use local Markdown; context persists with the note; instructions clear on leaving their mode; LLM transformation remains a separate explicit action.

**Alternatives considered**: Persisting Instruction text or adding tables/images was rejected because both exceed the confirmed scope.

## Technical Findings

- Frontend dependencies and scripts are defined in `package.json`: TypeScript 5.9, Vite 7, ProseMirror, and Vitest 4.
- The native desktop app uses Tauri 2 and Rust 2024 (`src-tauri/Cargo.toml`).
- The Tauri window already has a native title and standard decorated window at 1000×760 with a 640×480 minimum (`src-tauri/tauri.conf.json`); the frontend should not duplicate title-bar controls.
- `Toolbar.ts` currently combines mode and format controls with a consent checkbox and start/resume/pause/stop buttons.
- `SessionStatus.ts` currently includes a Pause button and warning; it should become a read-only status component for this feature.
- `DesktopShell.ts` renders the title, menus, notices, toolbar, status, note fields, editor, and preferences as a vertical flow; the new desktop shell will reorganize these existing elements.
- The native command handler currently receives `set_dictation_mode` separately from `start`; `NativeCapture::start_explicit` remains guarded by provider/configuration and OS lifecycle checks.
- `NoteCore` captures the dictation target at segment start and delays leaving Instructions while pending instruction segments are finalized.
- Manual content proposals are currently accepted only in `mode == "manual"`, and the frontend disables content editing during voice mode. Supporting edits while listening requires coordinated changes and concurrency tests.
- No `.specify/extensions.yml` exists; no before/after planning hooks are registered.

## Open Questions

None. Capture remains automatic after the user selects a mode, but application startup itself leaves all modes unselected and the microphone stopped.
