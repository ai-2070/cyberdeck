/**
 * Typed channels — strongly typed pub/sub over named channels.
 */

import type { Net as NapiNet } from '@ai-2070/net';
import type { SubscribeOpts, StoredEvent } from './types';
import { EventStream, TypedEventStream } from './stream';

/**
 * A strongly typed channel for publishing and subscribing to events.
 *
 * @example
 * ```typescript
 * interface TemperatureReading {
 *   sensor_id: string;
 *   celsius: number;
 *   timestamp: number;
 * }
 *
 * const temps = node.channel<TemperatureReading>('sensors/temperature');
 * temps.publish({ sensor_id: 'A1', celsius: 22.5, timestamp: Date.now() });
 *
 * for await (const reading of temps.subscribe()) {
 *   console.log(`${reading.sensor_id}: ${reading.celsius}°C`);
 * }
 * ```
 */
export class TypedChannel<T> {
  private bus: NapiNet;
  private channelName: string;
  private validator?: (data: unknown) => T;

  constructor(bus: NapiNet, channelName: string, validator?: (data: unknown) => T) {
    this.bus = bus;
    this.channelName = channelName;
    this.validator = validator;
  }

  /** The channel name. */
  get name(): string {
    return this.channelName;
  }

  /**
   * Publish a typed event to this channel.
   *
   * The event is serialized to JSON with the channel name embedded.
   */
  publish(event: T): boolean {
    const payload = JSON.stringify({
      ...event as object,
      _channel: this.channelName,
    });
    return this.bus.ingestFire(payload);
  }

  /**
   * Publish a batch of typed events to this channel.
   * Returns the number of events successfully published.
   */
  publishBatch(events: T[]): number {
    const payloads = events.map((event) =>
      JSON.stringify({
        ...event as object,
        _channel: this.channelName,
      })
    );
    return this.bus.ingestBatchFire(payloads);
  }

  /**
   * Subscribe to typed events on this channel.
   *
   * Returns an async iterable that deserializes and optionally validates
   * each event.
   */
  subscribe(opts: SubscribeOpts = {}): TypedEventStream<T> {
    const filter = JSON.stringify({ path: '_channel', value: this.channelName });
    const mergedOpts: SubscribeOpts = {
      ...opts,
      filter: opts.filter ?? filter,
    };

    const parse = this.validator
      ? (raw: string) => this.validator!(JSON.parse(raw))
      : (raw: string) => JSON.parse(raw) as T;

    return new TypedEventStream<T>(this.bus, mergedOpts, parse);
  }

  /**
   * Subscribe to raw events on this channel.
   */
  subscribeRaw(opts: SubscribeOpts = {}): EventStream {
    const filter = JSON.stringify({ path: '_channel', value: this.channelName });
    const mergedOpts: SubscribeOpts = {
      ...opts,
      filter: opts.filter ?? filter,
    };
    return new EventStream(this.bus, mergedOpts);
  }
}
