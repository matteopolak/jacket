import assert from "node:assert/strict";
import { test } from "node:test";
import { LanguageClient, documentUri, pathFromUri } from "../web/scripting-runtime/lsp-client.mjs";
import { SourcePath, Workspace } from "../web/scripting-runtime/workspace.mjs";

test("parsed file identities reject escaping paths and unrepresentable names", () => {
  for (const name of ["../secret.jai", "/root.jai", "a/../../secret.jai", "x\\y.jai", "C:/extra.jai", "lib/name:part.jai", "x\0.jai", "", "\ud800", "x".repeat(4097)]) {
    assert.throws(() => SourcePath.parse(name));
  }
  assert.throws(() => new SourcePath("../secret.jai", SourcePath));
  assert.equal(SourcePath.parse("lib/./nested/../value.jai").name, "lib/value.jai");
  assert.equal(SourcePath.parse("資料/値.jai").name, "資料/値.jai");
});

test("canonical aliases cannot overwrite another file or the entry", () => {
  const workspace = new Workspace("main :: () -> int { return 42; }");
  workspace.add("lib/value.jai", "answer :: 42;");
  const before = workspace.snapshot();
  assert.throws(() => workspace.add("lib/./value.jai", "answer :: 0;"));
  assert.throws(() => workspace.add("lib/../main.jai", "main :: () {}"));
  assert.deepEqual(workspace.snapshot(), before);
  workspace.select("main.jai");
  assert.throws(() => workspace.removeSelected());
});

test("a running source snapshot is isolated from later editor changes", () => {
  const workspace = new Workspace('#load "helper.jai"; main :: () -> int { return answer(); }');
  workspace.add("helper.jai", "answer :: () -> int { return 42; }");
  const snapshot = workspace.snapshot();
  workspace.edit("answer :: () -> int { return 9; }");
  workspace.removeSelected();
  assert.equal(snapshot.files["helper.jai"], "answer :: () -> int { return 42; }");
  assert.equal(workspace.selected.path.name, "main.jai");
  assert.deepEqual(workspace.snapshot().files, {});
  assert.throws(() => { snapshot.files["helper.jai"] = "changed"; });
});

test("hierarchical explorer retains canonical folder and file identities", () => {
  const workspace = new Workspace("entry");
  workspace.add("lib/math/value.jai", "answer :: 42;");
  workspace.add("lib/text.jai", "name :: \"Jai\";");
  const tree = workspace.tree;
  assert.equal(tree.children[0].kind, "directory");
  assert.equal(tree.children[0].path, "lib");
  assert.equal(tree.children[0].children[0].path, "lib/math");
  assert.equal(tree.children[0].children[0].children[0].path, "lib/math/value.jai");
  workspace.select("main.jai"); const before = workspace.documents[0].version;
  workspace.edit("entry"); assert.equal(workspace.documents[0].version, before);
  workspace.edit("edited"); assert.ok(workspace.documents[0].version > before);
  assert.equal(workspace.snapshot().source, "edited");
});

class LanguageWorker extends EventTarget {
  sent = [];
  postMessage(value) { this.sent.push(value); }
  receive(messages) { this.dispatchEvent(new MessageEvent("message", { data: { type: "lsp", messages } })); }
}

test("file URI identity matches canonical server escapes for punctuation and Unicode", () => {
  const path = "lib/bang!'()*/資料 file.jai";
  assert.equal(documentUri(path), "file:///jai-script/lib/bang%21%27%28%29%2A/%E8%B3%87%E6%96%99%20file.jai");
  assert.equal(pathFromUri(documentUri(path)), path);
  const worker = new LanguageWorker(), reports = [];
  const client = new LanguageClient(worker, { diagnostics: value => reports.push(value) });
  client.sync([{ path, text: "broken ::", version: 1 }]);
  worker.receive([{ method: "textDocument/publishDiagnostics", params: { uri: documentUri(path), version: 1, diagnostics: [{ message: "actual syntax error" }] } }]);
  assert.equal(reports.length, 1);
  client.dispose();
});

test("a rejected document notification disables stale-source navigation and pending requests", async () => {
  const worker = new LanguageWorker(), failures = [], reports = [];
  const client = new LanguageClient(worker, { failure: value => failures.push(value), diagnostics: value => reports.push(value) });
  const uri = documentUri("main.jai");
  client.sync([{ path: "main.jai", text: "answer :: 42;", version: 1 }]);
  client.sync([{ path: "main.jai", text: "x".repeat(256 * 1024 + 1), version: 2 }]);
  const pending = client.request("textDocument/hover", { textDocument: { uri }, position: { line: 0, character: 1 } });
  const rejection = assert.rejects(pending, /document byte budget exceeded/);
  const id = worker.sent.at(-1).message.id;
  // This is the core's actual notification-error wire form: requests carry errors,
  // but didChange rejection is a window/logMessage and leaves old source retained.
  worker.receive([{ jsonrpc: "2.0", id, result: { contents: "obsolete source" } }, { jsonrpc: "2.0", method: "window/logMessage", params: { type: 1, message: "document byte budget exceeded" } }]);
  await rejection;
  assert.deepEqual(failures, ["document byte budget exceeded"]);
  await assert.rejects(client.request("textDocument/definition", {}), /Language service stopped/);
  const count = worker.sent.length;
  client.sync([{ path: "main.jai", text: "small again", version: 3 }]);
  client.notify("initialized", {});
  assert.equal(worker.sent.length, count);
  worker.receive([{ method: "textDocument/publishDiagnostics", params: { uri, version: 2, diagnostics: [] } }]);
  assert.equal(reports.length, 0);
});

test("informational server logs preserve a healthy language session", async () => {
  const worker = new LanguageWorker(), failures = [];
  const client = new LanguageClient(worker, { failure: value => failures.push(value) });
  worker.receive([{ method: "window/logMessage", params: { type: 3, message: "information" } }]);
  const pending = client.request("textDocument/hover", {});
  const id = worker.sent.at(-1).message.id;
  worker.receive([{ jsonrpc: "2.0", id, result: null }]);
  assert.equal(await pending, null); assert.deepEqual(failures, []);
  client.dispose();
});
