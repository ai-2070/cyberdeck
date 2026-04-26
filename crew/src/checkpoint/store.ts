import type { CheckpointStore } from "./types.js";
import type { CrewSnapshot } from "../session/types.js";

export function createInMemoryCheckpointStore(): CheckpointStore {
  const map = new Map<string, CrewSnapshot>();
  return {
    put: (id, snapshot) => {
      map.set(id, snapshot);
    },
    get: (id) => map.get(id),
    list: () => [...map.keys()],
    delete: (id) => {
      map.delete(id);
    },
  };
}
