import { Channel, invoke, isTauri } from '@tauri-apps/api/core';
import type { Aggregate, EmptyPayload, OpenWorkspace, ResultEnvelope, RuntimeStatus, Snapshot, Subscribe, SubscriptionAck, TradeXError, Workspace } from '../shared/ipc-types.ts';
import { decode } from './projection.ts';

interface Inputs {
  'workspace.open': OpenWorkspace;
  'runtime.status': EmptyPayload;
  'domain.snapshot': Aggregate;
  'domain.subscribe': Subscribe;
}
interface Outputs {
  'workspace.open': Workspace;
  'runtime.status': RuntimeStatus;
  'domain.snapshot': Snapshot;
  'domain.subscribe': SubscriptionAck;
}
const definitions = {
  'workspace.open': ['OpenWorkspace', 'Workspace'],
  'runtime.status': ['EmptyPayload', 'RuntimeStatus'],
  'domain.snapshot': ['Aggregate', 'Snapshot'],
  'domain.subscribe': ['Subscribe', 'SubscriptionAck'],
} as const;

export const browserIntegration = import.meta.env.MODE === 'integration';
export const desktop = isTauri();
export const transportAvailable = desktop || browserIntegration;

export class CommandError extends Error {
  detail: TradeXError;
  constructor(detail: TradeXError) { super(detail.message); this.name = 'CommandError'; this.detail = detail; }
}

export async function request<C extends keyof Inputs>(command: C, payload: Inputs[C], onEvent?: (event: unknown) => void): Promise<Outputs[C]> {
  decode(definitions[command][0], payload);
  const envelope = { requestId: crypto.randomUUID(), schemaVersion: 1 as const, command, payload };
  let raw: unknown;
  if (desktop) {
    const events = new Channel<unknown>();
    events.onmessage = onEvent ?? (() => {});
    raw = await invoke('control', { request: envelope, events });
  } else if (browserIntegration) {
    const response = await fetch('/__integration/command', {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(envelope),
    });
    if (!response.ok) throw new Error('IPC_TRANSPORT_UNAVAILABLE');
    raw = await response.json();
  } else throw new Error('DESKTOP_REQUIRED');
  const result = decode<ResultEnvelope>('ResultEnvelope', raw);
  if (result.requestId !== envelope.requestId) throw new Error('IPC_REQUEST_MISMATCH');
  if (!result.ok) throw new CommandError(result.error);
  return decode<Outputs[C]>(definitions[command][1], result.data);
}

export async function subscribe(payload: Subscribe, onEvent: (event: unknown) => void, onError: (error: unknown) => void): Promise<() => void> {
  let active = true;
  if (desktop) {
    await request('domain.subscribe', payload, event => { if (active) onEvent(event); });
    return () => { active = false; };
  }
  if (!browserIntegration) throw new Error('DESKTOP_REQUIRED');
  const stream = new EventSource('/__integration/events');
  try {
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error('IPC_TRANSPORT_UNAVAILABLE')), 5000);
      stream.onopen = () => { clearTimeout(timeout); resolve(); };
      stream.onerror = () => { clearTimeout(timeout); reject(new Error('IPC_TRANSPORT_UNAVAILABLE')); };
    });
    stream.onmessage = message => {
      if (active) {
        try { onEvent(JSON.parse(message.data)); } catch (error) { onError(error); }
      }
    };
    stream.onerror = () => { if (active) { stream.close(); onError(new Error('IPC_TRANSPORT_UNAVAILABLE')); } };
    await request('domain.subscribe', payload);
    return () => { active = false; stream.close(); };
  } catch (error) { stream.close(); throw error; }
}

export function explainError(error: unknown): string {
  if (error instanceof CommandError) return error.message;
  if (error instanceof Error && error.message === 'IPC_SCHEMA_INCOMPATIBLE') return 'The application and runtime are incompatible. Update them together before continuing.';
  if (error instanceof Error && error.message.startsWith('IPC_SEQUENCE')) return 'Updates were interrupted. Reload the workspace to recover authoritative state.';
  return 'The local control plane is unavailable. Reopen TradeX or retry when it is available.';
}
