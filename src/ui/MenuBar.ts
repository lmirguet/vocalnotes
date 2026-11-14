import type { DocumentDispatch } from "./DocumentCommands";

// The menu itself is exclusively native. Tests inject this event-only adapter;
// it has no HTML menu, file picker, free path or alternative command authority.
export type DesktopAdapter = {
  listen: (callback: (dispatch: DocumentDispatch) => void) => Promise<() => void>;
};
