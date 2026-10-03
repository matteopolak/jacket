import assert from "node:assert/strict";
import { test } from "node:test";
import { StringStream } from "@codemirror/language";
import { EditorState } from "@codemirror/state";
import { history, undo, redo } from "@codemirror/commands";
import { jaiTokenizer } from "./browser-editor/jai-language.mjs";
import { LanguageClient, documentUri, pathFromUri, positionAt, offsetAt } from "../web/scripting-runtime/lsp-client.mjs";
function lex(text) {
  const state = jaiTokenizer.startState(), tokens = [];
  for (const line of text.split("\n")) {
    const stream = new StringStream(line, 4, 2);
    while (!stream.eol()) { stream.start = stream.pos; const style = jaiTokenizer.token(stream, state); assert.ok(stream.pos > stream.start); tokens.push([stream.current(), style]); }
  }
  return { state, tokens };
}
test("Jai lexical state retains nested comments, multiline strings and escaped quotes", () => {
  const { state, tokens } = lex('/* outer\n /* inner */ still outer */ main :: () -> int { return "/* \\" text\nend"; }');
  assert.equal(state.commentDepth, 0); assert.equal(state.string, false);
  assert.ok(tokens.some(([text, style]) => text.includes("inner") && style === "comment"));
  assert.ok(tokens.some(([text, style]) => text === "main" && style === "procedureName"));
  assert.ok(tokens.some(([text, style]) => text === "int" && style === "typeName"));
});
test("here string comments are text and a terminator prefix cannot close it", () => {
  const { state, tokens } = lex('#string,strip TAG\n// literal text\nTAG_suffix\nTAG; #load "helper.jai"');
  assert.equal(state.hereTag, null);
  assert.ok(tokens.some(([text, style]) => text === "// literal text" && style === "string"));
  assert.ok(tokens.some(([text, style]) => text === "TAG_suffix" && style === "string"));
  assert.ok(tokens.some(([text, style]) => text === "#load" && style === "directive"));
});
test("lexer retains incomplete editing states and ignores brackets inside strings/comments", () => {
  assert.equal(lex('/* not closed').state.commentDepth, 1);
  assert.equal(lex('"not closed').state.string, true);
  assert.equal(lex('{ "}" /* { */').state.depth, 1);
  assert.equal(jaiTokenizer.indent({ depth: 2 }, "}", { unit: 4 }), 4);
});
test("established editor history supports independent undo and redo", () => {
  let state = EditorState.create({ doc: "return 1;", extensions: [history()] });
  const target = { get state() { return state; }, dispatch(transaction) { state = transaction.state; } };
  state = state.update({ changes: { from: 7, to: 8, insert: "42" } }).state;
  assert.ok(undo(target)); assert.equal(state.doc.toString(), "return 1;");
  assert.ok(redo(target)); assert.equal(state.doc.toString(), "return 42;");
});
test("LSP positions use UTF-16 and reject unadmitted URIs and split surrogate boundaries", () => {
  const text = "a😀b\nnext";
  assert.deepEqual(positionAt(text, 4), { line: 0, character: 4 });
  assert.equal(offsetAt(text, { line: 1, character: 2 }), 7);
  assert.throws(() => offsetAt(text, { line: 0, character: 2 }));
  assert.throws(() => offsetAt(text, { line: 9, character: 0 }));
  assert.equal(pathFromUri(documentUri("lib/資料 file.jai")), "lib/資料 file.jai");
  assert.throws(() => pathFromUri("https://example.org/main.jai"));
  assert.throws(() => pathFromUri("file:///jai-script/%2e%2e/secret"));
});
class Worker extends EventTarget {
  sent = [];
  postMessage(value) { this.sent.push(value); }
  receive(data) { this.dispatchEvent(new MessageEvent("message", { data })); }
}
test("language transport preserves JSON-RPC IDs, atomic document versions and stale diagnostic rejection", async () => {
  const worker = new Worker(), reports = []; const client = new LanguageClient(worker, { diagnostics: value => reports.push(value) });
  const initialize = client.initialize(); const request = worker.sent.at(-1);
  worker.receive({ type: "lsp", id: request.id, messages: [{ jsonrpc: "2.0", id: request.message.id, result: { capabilities: {} } }] }); await initialize;
  client.sync([{ path: "main.jai", text: "main :: () {}", version: 1 }]);
  client.sync([{ path: "main.jai", text: "main :: () { return; }", version: 2 }]);
  const uri = documentUri("main.jai");
  worker.receive({ type: "lsp", messages: [{ method: "textDocument/publishDiagnostics", params: { uri, version: 1, diagnostics: [] } }] });
  worker.receive({ type: "lsp", messages: [{ method: "textDocument/publishDiagnostics", params: { uri, version: 2, diagnostics: [] } }] });
  assert.equal(reports.length, 1); assert.equal(reports[0].version, 2);
  client.sync([]); assert.equal(worker.sent.at(-1).message.method, "textDocument/didClose");
  const controller = new AbortController(); const pending = client.request("textDocument/hover", {}, controller.signal); controller.abort();
  await assert.rejects(pending, { name: "AbortError" }); assert.equal(worker.sent.at(-1).message.method, "$/cancelRequest"); client.dispose();
});
