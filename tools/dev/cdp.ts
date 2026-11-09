// A minimal Chrome DevTools Protocol client over Node's built-in WebSocket, shared by the doc probes.
import assert from 'node:assert/strict';

/** One protocol message: a reply (`id` with `result` or `error`) or an event (`method` with `params`). */
export interface CdpMessage {
  id?: number;
  method?: string;
  result?: unknown;
  error?: unknown;
  params?: {
    requestId?: string;
    resourceType?: string;
    request?: { url: string };
    exceptionDetails?: { text: string; exception?: { description?: string } };
  };
}

interface Pending {
  resolve(value: unknown): void;
  reject(error: Error): void;
  timer: NodeJS.Timeout;
  method: string;
}

/** The text of a `Runtime.exceptionThrown` event, as the probes report it. */
export function exceptionText(message: CdpMessage): string {
  return (
    message.params?.exceptionDetails?.exception?.description ||
    message.params?.exceptionDetails?.text ||
    ''
  );
}

/** A DevTools WebSocket (a browser or a page target).
 *
 *   const page = await new Connection(target.webSocketDebuggerUrl).open();
 *   page.listeners.push((m) => { if (m.method === 'Page.loadEventFired') loads++; });
 *   await page.cdp('Page.enable');
 *   const title = await page.evaluate<string>('document.title');
 *   page.close();
 *
 * Calls time out after 60 s. `listeners` receive every event, not the replies.
 */
export class Connection {
  url: string;
  next = 0;
  pending = new Map<number, Pending>();
  listeners: ((message: CdpMessage) => void)[] = [];
  socket: WebSocket | undefined;
  constructor(url: string) {
    this.url = url;
  }
  async open() {
    const socket = new WebSocket(this.url);
    this.socket = socket;
    await new Promise<void>((resolve, reject) => {
      socket.addEventListener('open', () => resolve(), { once: true });
      socket.addEventListener('error', reject, { once: true });
    });
    socket.addEventListener('message', (event) => {
      const message: CdpMessage = JSON.parse(String(event.data));
      const entry = message.id === undefined ? undefined : this.pending.get(message.id);
      if (entry && message.id !== undefined) {
        this.pending.delete(message.id);
        clearTimeout(entry.timer);
        if (message.error)
          entry.reject(new Error(`${entry.method}: ${JSON.stringify(message.error)}`));
        else entry.resolve(message.result);
      } else if (message.method) for (const l of this.listeners) l(message);
    });
    return this;
  }
  cdp<T = unknown>(method: string, params: Record<string, unknown> = {}): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      const id = ++this.next;
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`${method} timed out`));
      }, 60_000);
      this.pending.set(id, { resolve: resolve as Pending['resolve'], reject, timer, method });
      this.socket?.send(JSON.stringify({ id, method, params }));
    });
  }
  /** Run `expression` in the page (awaiting a promise) and return its value; a thrown error fails. */
  async evaluate<T = unknown>(expression: string): Promise<T> {
    const result = await this.cdp<{ result: { value: T }; exceptionDetails?: unknown }>(
      'Runtime.evaluate',
      { expression, awaitPromise: true, returnByValue: true },
    );
    assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails));
    return result.result.value;
  }
  close() {
    this.socket?.close();
  }
}
