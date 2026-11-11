import type { Snapshot } from "./contracts";

export type RenderEvidence = {
  type: "transcript-partial" | "note-commit";
  noteId: number;
  sessionId: number;
  generation: number;
  passageId: number;
  renderedAt: number;
};

// Two animation frames delimit a browser paint opportunity, not speech onset/end.
export function createRenderEvidence(target: HTMLElement, type: RenderEvidence["type"]) {
  let scope: [number, number, number] | undefined;
  const emitted = new Set<number>();
  const pending = new Map<number, number>();
  let latestPassage = 0;
  let disposed = false;
  function cancel() {
    for (const frame of pending.values()) window.cancelAnimationFrame(frame);
    pending.clear();
  }
  function accept(snapshot: Snapshot) {
    if (disposed) return false;
    const next = [snapshot.noteId, snapshot.sessionId, snapshot.generation];
    if (!next.every(value => Number.isSafeInteger(value) && value! >= 0)) return false;
    const current = next as [number, number, number];
    if (scope) {
      const difference = current.findIndex((value, index) => value !== scope![index]);
      if (difference >= 0 && current[difference]! < scope[difference]!) return false;
      if (difference < 0) return true;
    }
    cancel(); emitted.clear(); latestPassage = 0; scope = current;
    return true;
  }
  return {
    accept,
    cancel,
    destroy() { cancel(); disposed = true; },
    queue(snapshot: Snapshot, passageId: number | undefined, visible: () => boolean) {
      if (!accept(snapshot) || !Number.isSafeInteger(passageId) || passageId! <= 0 ||
        emitted.has(passageId!) || pending.has(passageId!)) return;
      const id = passageId!;
      if (id < latestPassage) return;
      cancel(); latestPassage = id;
      const identifiers = { noteId: snapshot.noteId, sessionId: snapshot.sessionId!,
        generation: snapshot.generation, passageId: id };
      pending.set(id, window.requestAnimationFrame(() => {
        pending.set(id, window.requestAnimationFrame(() => {
          pending.delete(id);
          if (disposed || !target.isConnected || target.closest("[hidden]") ||
            document.visibilityState === "hidden" || !visible()) return;
          for (let node: HTMLElement | null = target; node; node = node.parentElement) {
            const style = window.getComputedStyle(node);
            if (style.display === "none" || style.visibility === "hidden") return;
          }
          emitted.add(id);
          target.dispatchEvent(new CustomEvent<RenderEvidence>("vocalnotes-render", {
            bubbles: true,
            detail: { type, ...identifiers, renderedAt: performance.timeOrigin + performance.now() },
          }));
        }));
      }));
    },
  };
}
