import { SourcePath } from "./workspace.mjs";
export function documentUri(path) {
  return `file:///jai-script/${SourcePath.parse(path).name.split("/").map(part => encodeURIComponent(part).replace(/[!'()*]/gu, character => `%${character.charCodeAt(0).toString(16).toUpperCase()}`)).join("/")}`;
}
export function pathFromUri(uri) {
  const url = new URL(uri);
  if (url.protocol !== "file:" || url.host || !url.pathname.startsWith("/jai-script/")) throw new TypeError("Language server returned a non-workspace URI.");
  return SourcePath.parse(decodeURIComponent(url.pathname.slice("/jai-script/".length))).name;
}
export function positionAt(text, offset) {
  if (!Number.isInteger(offset) || offset < 0 || offset > text.length) throw new RangeError("Invalid document offset.");
  const prefix = text.slice(0, offset); const line = prefix.lastIndexOf("\n");
  return { line: prefix.split("\n").length - 1, character: offset - line - 1 };
}
export function offsetAt(text, position) {
  if (!position || !Number.isInteger(position.line) || !Number.isInteger(position.character) || position.line < 0 || position.character < 0) throw new RangeError("Invalid document position.");
  let from = 0;
  for (let i = 0; i < position.line; i++) { const next = text.indexOf("\n", from); if (next < 0) throw new RangeError("Position outside document."); from = next + 1; }
  let end = text.indexOf("\n", from); if (end < 0) end = text.length;
  if (position.character > end - from) throw new RangeError("Position outside line.");
  const offset = from + position.character;
  if (offset > 0 && offset < text.length && /[\uD800-\uDBFF]/u.test(text[offset - 1]) && /[\uDC00-\uDFFF]/u.test(text[offset])) throw new RangeError("Position splits a surrogate pair.");
  return offset;
}
export class LanguageClient {
  #worker; #next = 0; #pending = new Map(); #opened = new Map(); #stopped = false;
  constructor(worker, { diagnostics = () => {}, failure = () => {} } = {}) {
    this.#worker = worker; this.diagnostics = diagnostics; this.failure = failure;
    worker.addEventListener("message", ({ data }) => {
      if (this.#stopped || data.type !== "lsp") return;
      if (data.error) { this.#fail(String(data.error)); return; }
      const messages = data.messages ?? [];
      const rejected = messages.find(message => message.method === "window/logMessage" && message.params?.type === 1);
      if (rejected) { this.#fail(String(rejected.params.message || "Language server rejected document synchronization.")); return; }
      for (const message of messages) {
        if (message.method === "textDocument/publishDiagnostics") {
          const expected = this.#opened.get(message.params?.uri);
          if (expected !== undefined && message.params.version === expected) this.diagnostics(message.params);
        } else if (Object.hasOwn(message, "id")) {
          const entry = this.#pending.get(message.id); if (!entry) continue;
          this.#pending.delete(message.id); clearTimeout(entry.timer);
          message.error ? entry.reject(new Error(message.error.message)) : entry.resolve(message.result);
        }
      }
    });
    worker.addEventListener("error", event => { this.#fail(event.message || "Language worker failed."); });
  }
  #fail(message) { if (this.#stopped) return; this.dispose(new Error(message)); this.failure(message); }
  #send(message, id = message.id ?? ++this.#next) { if (this.#stopped) return; this.#worker.postMessage({ type: "lsp", id, message: { jsonrpc: "2.0", ...message } }); }
  notify(method, params) { this.#send({ method, params }); }
  request(method, params, signal) {
    if (this.#stopped) return Promise.reject(new Error("Language service stopped."));
    const id = ++this.#next;
    return new Promise((resolve, reject) => {
      const cancel = () => { if (!this.#pending.has(id)) return; this.#pending.delete(id); clearTimeout(timer); this.notify("$/cancelRequest", { id }); reject(new DOMException("Request cancelled", "AbortError")); };
      const timer = setTimeout(cancel, 10000);
      this.#pending.set(id, { resolve: value => { signal?.removeEventListener("abort", cancel); resolve(value); }, reject: error => { signal?.removeEventListener("abort", cancel); reject(error); }, timer });
      if (signal?.aborted) { cancel(); return; }
      signal?.addEventListener("abort", cancel, { once: true });
      this.#send({ id, method, params });
    });
  }
  async initialize() {
    const result = await this.request("initialize", { processId: null, rootUri: "file:///jai-script/", capabilities: { general: { positionEncodings: ["utf-16"] }, textDocument: { publishDiagnostics: { versionSupport: true }, hover: { contentFormat: ["plaintext"] }, completion: { completionItem: { snippetSupport: false, documentationFormat: ["plaintext"] } } } } });
    this.notify("initialized", {}); return result;
  }
  sync(documents) {
    if (this.#stopped) return;
    const current = new Set();
    for (const { path, text, version } of documents) {
      const uri = documentUri(path); current.add(uri);
      const previous = this.#opened.get(uri);
      if (previous === version) continue;
      this.#opened.set(uri, version);
      if (previous === undefined) this.notify("textDocument/didOpen", { textDocument: { uri, languageId: "jai", version, text } });
      else this.notify("textDocument/didChange", { textDocument: { uri, version }, contentChanges: [{ text }] });
    }
    for (const uri of this.#opened.keys()) if (!current.has(uri)) {
      this.#opened.delete(uri); this.notify("textDocument/didClose", { textDocument: { uri } });
    }
  }
  dispose(error = new Error("Language service stopped.")) { this.#stopped = true; for (const entry of this.#pending.values()) { clearTimeout(entry.timer); entry.reject(error); } this.#pending.clear(); this.#opened.clear(); }
}
