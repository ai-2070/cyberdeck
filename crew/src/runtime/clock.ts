// The library never calls Date.now() directly. Callers inject a Clock; tests
// inject a frozen one for deterministic timestamps.

export interface Clock {
  now(): number;
}

export interface MutableClock extends Clock {
  advance(ms: number): void;
  set(t: number): void;
}

export function systemClock(): Clock {
  return { now: () => Date.now() };
}

export function frozenClock(initial = 0): MutableClock {
  let t = initial;
  return {
    now: () => t,
    advance: (ms: number) => {
      t += ms;
    },
    set: (newT: number) => {
      t = newT;
    },
  };
}
