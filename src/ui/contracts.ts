export type Block = { from: number; to: number; raw: string; kind: string };
export type DictationMode = "content" | "context" | "instructions";
export type DocumentKind = "new" | "open" | "save" | "save_as" | "export_markdown" | "quit";
export type DocumentPhase = "awaiting_ui" | "draining" | "choosing_file" | "confirming_save"
  | "confirming_overwrite" | "writing" | "committing" | "idle";
export type DocumentOperation = {
  operationId: string; action: DocumentKind; phase: DocumentPhase;
  noteId: number; cancellable: boolean;
};
export type DocumentError = {
  code: "forbidden" | "busy" | "stale_operation" | "unsynchronized_ui" | "stale_revision"
    | "invalid_path" | "unsupported_format" | "unsupported_storage" | "invalid_document"
    | "size_limit" | "external_conflict" | "write_failed" | "drain_failed"
    | "window_failed" | "dialog_failed" | "preferences_invalid"
    | "preferences_write_failed"     | "core_unavailable" | "history_unavailable";
  message: string;
};
export type HistoryAvailability = { canUndo: boolean; canRedo: boolean };
export type DocumentHistory = Record<DictationMode, HistoryAvailability>;
export type Snapshot = {
  noteId: number; manualPeriod: number; revision: number; generation: number;
  sessionId?: number; transcriptPassageId?: number; appliedPassageId?: number;
  speechStartedAtEpochUs?: number | null; speechEndedAtEpochUs?: number | null;
  appliedSpeechStartedAtEpochUs?: number | null; appliedSpeechEndedAtEpochUs?: number | null;
  listeningState?: "stopped" | "starting" | "listening";
  voiceActivity?: string;
  draftingIntention?: { purpose: string; form: string; tone: string; context: string } | null;
  changeFeedback?: { revision: number; areas: { from: number; to: number;
    kind: "insert" | "replace" | "delete" | "move"; removedCharacters: number }[] } | null;
  failureStage?: "recognition" | "interpretation" | "application" | null;
  mode: string; dictationMode: DictationMode; context: string; instructions: string;
  dictationTransitionPending: boolean; inputGeneration: number;
  source: string; dirty: boolean; message: string; blocks: Block[];
  documentName: string; documentPath: string | null; documentFormat: "von" | "markdown" | null;
  history: DocumentHistory;
  documentOperation: DocumentOperation | null; documentError: DocumentError | null;
  storageMessage?: string;
  preferencesMigrated?: boolean;
  sttRequests?: number; llmRequests?: number; priorityRequests?: number; audioMs?: number;
  sttModel?: string; llmModel?: string;
  sttConfiguredModel?: string; sttAcknowledgedModel?: string;
  lastSttFinalMs?: number | null; lastLlmMs?: number | null;
  lastPriorityMs?: number | null; lastCommitUs?: number | null;
  voiceApplied?: number; voiceClarifications?: number; voiceRejected?: number; voiceNoChange?: number;
  lastVoiceResult?: string;
  transcriptEnabled?: boolean;
  transcriptProvisional?: string; transcriptFinal?: string;
  closePending?: boolean;
};
export type Patch = { from: number; to: number; expected: string; text: string };
export type Proposal = {
  transactionId: string; noteId: number; manualPeriod: number;
  baseRevision: number; patches: Patch[];
};
export type Ack = { transactionId: string; accepted: boolean; message: string; snapshot: Snapshot };
export type Invoke = (command: string, args?: Record<string, unknown>) => Promise<unknown>;
