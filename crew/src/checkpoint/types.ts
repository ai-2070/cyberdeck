import type { CrewSnapshot } from "../session/types.js";

// Caller-pluggable persistence for snapshots. The library doesn't auto-write
// snapshots — the crewControl.checkpoint(id) hook emits a checkpoint.taken
// event, and observers (typically the bus consumer) capture session.snapshot()
// and store it under that id.

export interface CheckpointStore {
  put(id: string, snapshot: CrewSnapshot): void;
  get(id: string): CrewSnapshot | undefined;
  list(): string[];
  delete(id: string): void;
}
