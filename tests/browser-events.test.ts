import assert from 'node:assert/strict';
import test from 'node:test';
import { subscribeBrowserEvents } from '../src/browserEvents.ts';

test('eight subscriptions share one SSE, survive a sibling close, and recover after disconnect', async () => {
  const previous = globalThis.EventSource;
  class Stream {
    static created: Stream[] = [];
    onopen?: () => void;
    onmessage?: (event: { data: string }) => void;
    onerror?: () => void;
    closed = false;
    constructor() { Stream.created.push(this); queueMicrotask(() => this.onopen?.()); }
    close() { this.closed = true; }
  }
  globalThis.EventSource = Stream as unknown as typeof EventSource;
  const messages: string[][] = Array.from({ length: 8 }, () => []);
  const errors: unknown[] = [];
  const closes: (() => void)[] = [];
  try {
    closes.push(...await Promise.all(messages.map(items => subscribeBrowserEvents(data => items.push(data), error => errors.push(error)))));
    assert.equal(Stream.created.length, 1);
    const stream = Stream.created[0];
    stream.onmessage?.({ data: 'first' });
    assert.ok(messages.every(items => items[0] === 'first'));
    closes[0]();
    assert.equal(stream.closed, false);
    stream.onmessage?.({ data: 'second' });
    assert.deepEqual(messages[0], ['first']);
    assert.ok(messages.slice(1).every(items => items[1] === 'second'));
    stream.onerror?.();
    assert.equal(stream.closed, true);
    assert.equal(errors.length, 7);
    closes.push(await subscribeBrowserEvents(() => {}, error => errors.push(error)));
    assert.equal(Stream.created.length, 2);
    closes.at(-1)!();
    assert.equal(Stream.created[1].closed, true);
  } finally { closes.forEach(close => close()); globalThis.EventSource = previous; }
});
