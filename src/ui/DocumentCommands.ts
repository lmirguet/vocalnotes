import type { DictationMode, DocumentOperation, Invoke, Snapshot } from "./contracts";

export type DocumentDispatch = { type: "prepare"; dispatchId: string; operationId: string }
  | { type: "history"; dispatchId: string; direction: "undo" | "redo"; area: DictationMode }
  | { type: "restore_focus"; dispatchId: string; area: DictationMode };
type Reply = { snapshot: Snapshot };
type Options = {
  invoke: Invoke;
  listen: (callback: (dispatch: DocumentDispatch) => void) => Promise<() => void>;
  prepare: (operation: DocumentOperation) => Promise<Snapshot>;
  onSnapshot: (snapshot: Snapshot) => void;
  onError: (error: unknown) => void;
  onHistory?: (direction: "undo" | "redo", area: DictationMode) => Promise<void>;
  onRestoreFocus?: (area: DictationMode) => void;
};

function remember(set: Set<string>, id: string) {
  set.add(id);
  if (set.size > 256) {
    const first = set.values().next();
    if (!first.done) set.delete(first.value);
  }
}

export async function createDocumentCommands(options: Options) {
  let disposed = false;
  let current: Snapshot | undefined;
  let startQueue!: () => void;
  let queue = new Promise<void>(resolve => { startQueue = resolve; });
  const dispatches = new Set<string>();
  const acknowledged = new Set<string>();
  const blocked = new Set<string>();
  const preparing = new Set<string>();
  const finished = new Set<string>();

  function enqueue(work: () => Promise<void>) {
    queue = queue.then(async () => { if (!disposed) await work(); })
      .catch(error => { if (!disposed) options.onError(error); });
  }

  function update(snapshot: Snapshot) {
    if (disposed || (snapshot.documentOperation && finished.has(snapshot.documentOperation.operationId))) return false;
    if (current && (snapshot.noteId < current.noteId || (snapshot.noteId === current.noteId
      && (snapshot.revision < current.revision || snapshot.inputGeneration < current.inputGeneration)))) return false;
    if (current?.documentOperation && current.documentOperation.operationId !== snapshot.documentOperation?.operationId) {
      remember(finished, current.documentOperation.operationId);
    }
    current = snapshot;
    return true;
  }
  function accept(snapshot: Snapshot) {
    if (!update(snapshot)) return;
    options.onSnapshot(snapshot);
  }

  async function prepare(operation: DocumentOperation) {
    const id = operation.operationId;
    if (disposed || operation.phase !== "awaiting_ui"
      || acknowledged.has(id) || preparing.has(id) || blocked.has(id)) return;
    preparing.add(id);
    try {
      const prepared = await options.prepare(operation);
      if (disposed || current?.documentOperation?.operationId !== id
        || current.documentOperation.phase !== "awaiting_ui") return;
      if (prepared.noteId !== operation.noteId) {
        throw new Error("La note active a changé pendant la préparation.");
      }
      // An uncertain IPC response must not trigger a second execution.
      remember(acknowledged, id);
      const reply = await options.invoke("document_action", { action: {
        type: "ready", operationId: id, noteId: prepared.noteId,
        revision: prepared.revision, inputGeneration: prepared.inputGeneration,
      } }) as Reply;
      if (disposed || current?.documentOperation?.operationId !== id) return;
      accept(reply.snapshot);
    } catch (error) {
      if (typeof error === "object" && error !== null && "code" in error
        && (error.code === "unsynchronized_ui" || error.code === "stale_revision")) {
        acknowledged.delete(id);
        const snapshot = await options.invoke("prototype_snapshot") as Snapshot;
        accept(snapshot);
        if (snapshot.documentOperation?.operationId === id) {
          enqueue(() => prepare(snapshot.documentOperation!));
        }
        return;
      }
      remember(blocked, id);
      throw error;
    } finally {
      preparing.delete(id);
    }
  }

  function observe(snapshot: Snapshot) {
    if (!update(snapshot)) return;
    const operation = snapshot.documentOperation;
    if (operation?.phase === "awaiting_ui") enqueue(() => prepare(operation));
  }

  const unlisten = await options.listen(dispatch => {
    if (disposed || dispatches.has(dispatch.dispatchId)) return;
    remember(dispatches, dispatch.dispatchId);
    enqueue(async () => {
      if (dispatch.type === "history") {
        await options.onHistory?.(dispatch.direction, dispatch.area);
        return;
      }
      if (dispatch.type === "restore_focus") {
        options.onRestoreFocus?.(dispatch.area);
        return;
      }
      const snapshot = await options.invoke("prototype_snapshot") as Snapshot;
      if (disposed) return;
      accept(snapshot);
      const operation = snapshot.documentOperation;
      if (dispatch.type === "prepare" && operation?.operationId === dispatch.operationId) {
        await prepare(operation);
      }
    });
  });

  try {
    const snapshot = await options.invoke("prototype_snapshot") as Snapshot;
    accept(snapshot);
    observe(snapshot);
    startQueue();
  } catch (error) {
    disposed = true;
    unlisten();
    startQueue();
    throw error;
  }

  async function action(value: { type: "cancel"; operationId: string }
    | { type: "focus"; area: DictationMode }) {
    if (disposed) throw new Error("Le bridge documentaire est fermé.");
    // Cancellation must remain usable while prepare/ready is awaiting a result.
    const reply = await options.invoke("document_action", { action: value }) as Reply;
    accept(reply.snapshot);
    observe(reply.snapshot);
  }

  return {
    observe,
    cancel: (operationId: string) => action({ type: "cancel", operationId }),
    focus: (area: DictationMode) => action({ type: "focus", area }),
    retryPreparation() {
      const operation = current?.documentOperation;
      if (operation) {
        blocked.delete(operation.operationId);
        enqueue(() => prepare(operation));
      }
    },
    async settled() {
      let pending: Promise<void>;
      do { pending = queue; await pending; } while (pending !== queue);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      unlisten();
    },
  };
}
