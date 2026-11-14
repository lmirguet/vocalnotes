import { expect, it, vi } from "vitest";
import { createDocumentCommands, type DocumentDispatch } from "./DocumentCommands";
import type { DocumentOperation, Invoke, Snapshot } from "./contracts";

const operation: DocumentOperation = {
  operationId: "9007199254740992", action: "quit", phase: "awaiting_ui",
  noteId: 1, cancellable: true,
};
const initial: Snapshot = {
  noteId: 1, manualPeriod: 1, revision: 3, generation: 1, inputGeneration: 5,
  mode: "manual", dictationMode: "content", dictationTransitionPending: false,
  context: "", instructions: "", source: "Texte non enregistré", dirty: true,
  message: "", blocks: [], documentName: "Sans titre", documentPath: null,
  documentFormat: null, documentOperation: null, documentError: null,
  history: { content: { canUndo: false, canRedo: false },
    context: { canUndo: false, canRedo: false }, instructions: { canUndo: false, canRedo: false } },
};

async function setup(pending = false) {
  let state: Snapshot = { ...initial, documentOperation: pending ? operation : null };
  let listener: ((dispatch: DocumentDispatch) => void) | undefined;
  const trace: string[] = [];
  const unlisten = vi.fn();
  const prepare = vi.fn(async (_operation: DocumentOperation) => state);
  const onSnapshot = vi.fn();
  const onError = vi.fn();
  const invoke = vi.fn<Invoke>(async (command, args) => {
    trace.push(command);
    if (command === "prototype_snapshot") return state;
    const action = args!.action as { type: string };
    if (action.type === "ready" || action.type === "cancel") {
      state = { ...state, documentOperation: null };
    }
    return { snapshot: state, outcome: null };
  });
  const bridge = await createDocumentCommands({
    invoke, prepare, onSnapshot, onError,
    listen: async callback => { trace.push("listen"); listener = callback; return unlisten; },
  });
  return {
    bridge, trace, prepare, invoke, onSnapshot, onError, unlisten,
    state: () => state,
    activate: () => { state = { ...state, documentOperation: operation }; },
    emit: (dispatchId = "dispatch-1") => listener!({
      type: "prepare", dispatchId, operationId: operation.operationId,
    }),
  };
}

it("installe le listener avant le snapshot et récupère une intention manquée sans la réémettre", async () => {
  const app = await setup(true);
  await app.bridge.settled();
  expect(app.trace.slice(0, 2)).toEqual(["listen", "prototype_snapshot"]);
  expect(app.prepare).toHaveBeenCalledOnce();
  const calls = app.invoke.mock.calls.filter(([name]) => name === "document_action");
  expect(calls.map(([, args]) => args!.action)).toEqual([{
    type: "ready", operationId: operation.operationId, noteId: 1, revision: 3, inputGeneration: 5,
  }]);
  expect(app.onError).not.toHaveBeenCalled();
  app.bridge.dispose();
});

it("fusionne dispatchs répétés et snapshots répétés pour une même opération", async () => {
  const app = await setup();
  app.activate();
  app.emit(); app.emit(); app.emit("dispatch-2");
  app.bridge.observe(app.state());
  await app.bridge.settled();
  expect(app.prepare).toHaveBeenCalledOnce();
  expect(app.invoke.mock.calls.filter(([name]) => name === "document_action")).toHaveLength(1);
  app.bridge.dispose();
});

it("ne prépare jamais une opération déjà en écriture", async () => {
  const app = await setup();
  app.bridge.observe({ ...initial, documentOperation: { ...operation, phase: "writing", cancellable: false } });
  await app.bridge.settled();
  expect(app.prepare).not.toHaveBeenCalled();
  app.bridge.dispose();
});

it("signale les saisies non réconciliées sans envoyer ready et permet une reprise explicite", async () => {
  const app = await setup();
  const error = new Error("Saisie récupérable non résolue.");
  app.prepare.mockRejectedValueOnce(error);
  app.activate(); app.emit();
  await app.bridge.settled();
  expect(app.onError).toHaveBeenCalledWith(error);
  expect(app.invoke.mock.calls.filter(([name]) => name === "document_action")).toHaveLength(0);
  app.bridge.observe(app.state());
  await app.bridge.settled();
  expect(app.prepare).toHaveBeenCalledOnce();
  app.bridge.retryPreparation();
  await app.bridge.settled();
  expect(app.prepare).toHaveBeenCalledTimes(2);
  expect(app.invoke.mock.calls.filter(([name]) => name === "document_action")).toHaveLength(1);
  app.bridge.dispose();
});

it("annule sans attendre une préparation bloquée et n'envoie pas de ready tardif", async () => {
  const app = await setup();
  let release!: (snapshot: Snapshot) => void;
  app.prepare.mockImplementationOnce(() => new Promise(resolve => { release = resolve; }));
  app.activate(); app.emit();
  await vi.waitFor(() => expect(app.prepare).toHaveBeenCalledOnce());
  await app.bridge.cancel(operation.operationId);
  release(app.state());
  await app.bridge.settled();
  expect(app.invoke.mock.calls.filter(([name]) => name === "document_action")
    .map(([, args]) => args!.action)).toEqual([{ type: "cancel", operationId: operation.operationId }]);
  app.bridge.dispose();
});

it("désabonne une seule fois et ne poursuit pas une préparation après dispose", async () => {
  const app = await setup();
  let release!: (snapshot: Snapshot) => void;
  app.prepare.mockImplementationOnce(() => new Promise(resolve => { release = resolve; }));
  app.activate(); app.emit();
  await vi.waitFor(() => expect(app.prepare).toHaveBeenCalledOnce());
  app.bridge.dispose(); app.bridge.dispose();
  release(app.state());
  await app.bridge.settled();
  expect(app.unlisten).toHaveBeenCalledOnce();
  expect(app.invoke.mock.calls.filter(([name]) => name === "document_action")).toHaveLength(0);
});

it("ignore une réponse ready arrivée après l'acquittement d'une annulation", async () => {
  const app = await setup();
  const original = app.invoke.getMockImplementation()!;
  let release!: () => void;
  app.invoke.mockImplementation((command, args) => {
    if (command === "document_action" && (args!.action as { type: string }).type === "ready") {
      return new Promise(resolve => {
        release = () => resolve({ snapshot: { ...initial,
          documentOperation: { ...operation, phase: "draining" } } });
      });
    }
    return original(command, args);
  });
  app.activate(); app.emit();
  await vi.waitFor(() => expect(release).toBeTypeOf("function"));
  await app.bridge.cancel(operation.operationId);
  const notifications = app.onSnapshot.mock.calls.length;
  release();
  await app.bridge.settled();
  expect(app.onSnapshot).toHaveBeenCalledTimes(notifications);
  expect(app.onSnapshot.mock.lastCall![0].documentOperation).toBeNull();
  app.bridge.dispose();
});

it("transmet la dernière zone active sans envoyer son texte", async () => {
  const app = await setup();
  await app.bridge.focus("context");
  expect(app.invoke).toHaveBeenCalledWith("document_action", { action: { type: "focus", area: "context" } });
  app.bridge.dispose();
});

it("autorise une nouvelle préparation après un refus explicite de révision par Rust", async () => {
  const app = await setup();
  const original = app.invoke.getMockImplementation()!;
  let refused = false;
  app.invoke.mockImplementation(async (command, args) => {
    if (command === "document_action" && !refused) {
      refused = true;
      throw { code: "unsynchronized_ui", message: "Réconcilier les saisies." };
    }
    return original(command, args);
  });
  app.activate(); app.emit();
  await app.bridge.settled();
  expect(app.onError).not.toHaveBeenCalled();
  expect(app.prepare).toHaveBeenCalledTimes(2);
  expect(app.state().documentOperation).toBeNull();
  app.bridge.dispose();
});

it("attend le snapshot initial avant de traiter un événement arrivé pendant l'abonnement", async () => {
  let release!: (snapshot: Snapshot) => void;
  const snapshot = { ...initial, documentOperation: operation };
  const prepare = vi.fn(async () => snapshot);
  const onError = vi.fn();
  let reads = 0;
  const creating = createDocumentCommands({
    invoke: async command => {
      if (command === "prototype_snapshot") {
        if (++reads === 1) return new Promise<Snapshot>(resolve => { release = resolve; });
        return snapshot;
      }
      return { snapshot: { ...snapshot, documentOperation: null } };
    },
    prepare, onSnapshot: vi.fn(), onError,
    listen: async callback => {
      callback({ type: "prepare", dispatchId: "early", operationId: operation.operationId });
      return vi.fn<() => void>();
    },
  });
  await vi.waitFor(() => expect(reads).toBe(1));
  expect(prepare).not.toHaveBeenCalled();
  release(snapshot);
  const bridge = await creating;
  await bridge.settled();
  expect(prepare).toHaveBeenCalledOnce();
  expect(onError).not.toHaveBeenCalled();
  bridge.dispose();
});
