/**
 * Async streaming event consumption.
 */

import type { Net as NapiNet } from '@ai2070/net';
import type { StoredEvent, SubscribeOpts } from './types';

const DEFAULT_POLL_INTERVAL = 1;
const DEFAULT_MAX_BACKOFF = 100;
const DEFAULT_LIMIT = 100;

/**
 * An async iterable stream of events from the bus.
 *
 * Uses adaptive polling — tight loop when events flow, exponential
 * backoff when idle.
 *
 * @example
 * ```typescript
 * for await (const event of node.subscribe({ limit: 100 })) {
 *   console.log(event.raw);
 * }
 * ```
 */
export class EventStream implements AsyncIterable<StoredEvent> {
  private bus: NapiNet;
  private opts: Required<SubscribeOpts>;
  private cursor?: string;
  private aborted = false;

  constructor(bus: NapiNet, opts: SubscribeOpts = {}) {
    this.bus = bus;
    this.opts = {
      limit: opts.limit ?? DEFAULT_LIMIT,
      filter: opts.filter ?? '',
      ordering: opts.ordering ?? 'none',
      pollIntervalMs: opts.pollIntervalMs ?? DEFAULT_POLL_INTERVAL,
      maxBackoffMs: opts.maxBackoffMs ?? DEFAULT_MAX_BACKOFF,
    };
  }

  /** Stop the stream. */
  stop(): void {
    this.aborted = true;
  }

  async *[Symbol.asyncIterator](): AsyncIterableIterator<StoredEvent> {
    let backoff = this.opts.pollIntervalMs;

    while (!this.aborted) {
      const response = await this.bus.poll({
        limit: this.opts.limit,
        cursor: this.cursor,
        filter: this.opts.filter || undefined,
        ordering: this.opts.ordering,
      });

      if (response.events.length > 0) {
        backoff = this.opts.pollIntervalMs;
        this.cursor = response.nextId ?? undefined;

        for (const event of response.events) {
          yield {
            id: event.id,
            raw: event.raw,
            insertionTs: event.insertionTs,
            shardId: event.shardId,
          };
        }
      } else {
        // Exponential backoff when idle.
        await sleep(backoff);
        backoff = Math.min(backoff * 2, this.opts.maxBackoffMs);
      }
    }
  }
}

/**
 * A typed async iterable stream that deserializes events into `T`.
 *
 * @example
 * ```typescript
 * interface TokenEvent { token: string; index: number; }
 * for await (const token of node.subscribe<TokenEvent>({ limit: 100 })) {
 *   console.log(token.token, token.index);
 * }
 * ```
 */
export class TypedEventStream<T> implements AsyncIterable<T> {
  private inner: EventStream;
  private parse: (raw: string) => T;

  constructor(bus: NapiNet, opts: SubscribeOpts = {}, parse?: (raw: string) => T) {
    this.inner = new EventStream(bus, opts);
    this.parse = parse ?? ((raw: string) => JSON.parse(raw) as T);
  }

  /** Stop the stream. */
  stop(): void {
    this.inner.stop();
  }

  async *[Symbol.asyncIterator](): AsyncIterableIterator<T> {
    for await (const event of this.inner) {
      yield this.parse(event.raw);
    }
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
