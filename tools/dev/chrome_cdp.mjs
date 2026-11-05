/** Minimal shared CDP transport; callers own browser/profile lifetimes and every event assertion. */
export class Connection {
  constructor(url) { this.url = url; this.next = 0; this.pending = new Map(); this.listeners = []; }
  async open() {
    this.socket = new WebSocket(this.url);
    await new Promise((resolve, reject) => { this.socket.onopen = resolve; this.socket.onerror = reject; });
    this.socket.onmessage = event => {
      const message = JSON.parse(event.data);
      const entry = this.pending.get(message.id);
      if (entry) {
        this.pending.delete(message.id); clearTimeout(entry.timer);
        if (message.error) entry.reject(new Error(`${entry.method}: ${JSON.stringify(message.error)}`)); else entry.resolve(message.result);
      } else if (message.method) for (const l of this.listeners) l(message);
    };
    return this;
  }
  cdp(method, params = {}, sessionId) {
    return new Promise((resolve, reject) => {
      const id = ++this.next;
      const timer = setTimeout(() => { this.pending.delete(id); reject(new Error(`${method} timed out`)); }, 60_000);
      this.pending.set(id, { resolve, reject, timer, method });
      this.socket.send(JSON.stringify({ id, method, params, sessionId }));
    });
  }
  close() { this.socket?.close(); }
}


/** Preserve every Runtime exception; transition aborts are evidence, never a text-filter exemption. */
export function runtimeException(message) {
  if (message.method !== 'Runtime.exceptionThrown') return undefined;
  const details = message.params.exceptionDetails;
  return details.exception?.description || details.text || JSON.stringify(details);
}
