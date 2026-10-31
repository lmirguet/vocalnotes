export type Block = { from: number; to: number; raw: string; kind: string };
export type Snapshot = {
  noteId: number; manualPeriod: number; revision: number; generation: number;
  mode: string; source: string; dirty: boolean; message: string; blocks: Block[];
  storageMessage?: string;
  sttRequests?: number; llmRequests?: number; priorityRequests?: number; audioMs?: number;
  sttModel?: string; llmModel?: string;
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
