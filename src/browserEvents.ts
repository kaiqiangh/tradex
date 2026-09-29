// Integration-only transport. One SSE connection leaves browser HTTP slots for commands.
type Listener = { event: (data: string) => void; error: (error: unknown) => void };
let connection: { stream: EventSource; opened: Promise<void>; listeners: Set<Listener> } | undefined;

export async function subscribeBrowserEvents(event: Listener['event'], error: Listener['error']): Promise<() => void> {
  if (!connection) {
    const stream = new EventSource('/__integration/events');
    const listeners = new Set<Listener>();
    const opened = new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => { stream.close(); reject(new Error('IPC_TRANSPORT_UNAVAILABLE')); }, 5000);
      stream.onopen = () => { clearTimeout(timeout); resolve(); };
      stream.onerror = () => {
        clearTimeout(timeout); stream.close();
        if (connection?.stream === stream) connection = undefined;
        const failure = new Error('IPC_TRANSPORT_UNAVAILABLE');
        reject(failure);
        for (const listener of listeners) listener.error(failure);
      };
    });
    stream.onmessage = message => { for (const listener of listeners) listener.event(message.data); };
    connection = { stream, opened, listeners };
  }
  const current = connection;
  const listener = { event, error };
  current.listeners.add(listener);
  const close = () => {
    current.listeners.delete(listener);
    if (!current.listeners.size) {
      current.stream.close();
      if (connection === current) connection = undefined;
    }
  };
  try { await current.opened; return close; }
  catch (failure) { close(); throw failure; }
}
