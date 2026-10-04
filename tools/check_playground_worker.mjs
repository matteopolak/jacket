import assert from "node:assert/strict";
import { Worker } from "node:worker_threads";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
export async function checkWorkers(directory) {
  const workers = new Set();
  function start() {
    const worker = new Worker(new URL("./playground_worker_host.mjs", import.meta.url), { workerData: { directory: path.resolve(directory) } });
    workers.add(worker); const pending = new Set();
    worker.on("message", data => { for (const item of pending) if (data.type === item.type && (item.id === undefined || data.id === item.id)) { pending.delete(item); clearTimeout(item.timer); item.resolve(data); } });
    worker.on("error", error => { for (const item of pending) { clearTimeout(item.timer); item.reject(error); } pending.clear(); });
    return { worker, send: value => worker.postMessage(value), wait(type, id) { return new Promise((resolve, reject) => { const item = { type, id, resolve, reject, timer: setTimeout(() => { pending.delete(item); reject(new Error(`Real worker timed out: ${type}`)); }, 20000) }; pending.add(item); }); } };
  }
  async function initialize(client) { const ready = client.wait("init"); client.send({ type: "init" }); const response = await ready; assert.equal(response.error, undefined); assert.equal(response.capabilities.languageServer, true); }
  // Workers answer with the jaic playground result (`play`).
  function exitCodeOf(message) { return String(message.play.exitCode); }
  async function run(client, id, source, options) { const finished = client.wait("run", id); client.send({ type: "run", id, source, options }); const result = await finished; assert.equal(result.error, undefined); return result.play; }
  try {
    const execution = start(), language = start(); await Promise.all([initialize(execution), initialize(language)]);
    const snapshot = { type: "run", id: 1, source: '#load "lib/helper.jai"; main :: () -> int { return answer(); }', options: { files: { "lib/helper.jai": "answer :: () -> int { return 42; }" }, fuel: 1000000 } };
    const finished = execution.wait("run", 1); execution.send(snapshot); snapshot.options.files["lib/helper.jai"] = "answer :: () -> int { return 9; }";
    assert.equal(exitCodeOf(await finished), "42", "Actual worker uses its immutable cloned VFS snapshot");
    let rpcId = 0;
    async function request(method, params) { const id = ++rpcId; const response = language.wait("lsp", id); language.send({ type: "lsp", id, message: { jsonrpc: "2.0", id, method, params } }); const envelope = await response; assert.equal(envelope.error, undefined); const message = envelope.messages.find(value => value.id === id); assert(message && !message.error); return message.result; }
    const initialized = await request("initialize", { capabilities: {} }); assert.equal(initialized.capabilities.hoverProvider, true);
    const uri = "file:///jai-script/main.jai";
    language.send({ type: "lsp", id: 100, message: { jsonrpc: "2.0", method: "initialized", params: {} } });
    const opened = language.wait("lsp", 101); language.send({ type: "lsp", id: 101, message: { jsonrpc: "2.0", method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "jai", version: 1, text: "answer :: () -> int { return 42; }\nmain :: () -> int { return answer(); }" } } } });
    const publication = await opened; assert.equal(publication.error, undefined); assert(publication.messages.some(value => value.method === "textDocument/publishDiagnostics" && value.params.version === 1));
    const started = execution.wait("run-started", 2); execution.send({ type: "run", id: 2, source: "main :: () { while true {} }", options: { fuel: 0xffffffff } }); await started;
    await execution.worker.terminate(); workers.delete(execution.worker);
    const hover = await request("textDocument/hover", { textDocument: { uri }, position: { line: 1, character: "main :: () -> int { return answer(); }".indexOf("answer") + 1 } }); assert.match(hover.contents.value, /answer/);
    const restarted = start(); await initialize(restarted); assert.equal(String((await run(restarted, 3, "main :: () -> int { return 42; }", { fuel: 1000000 })).exitCode), "42");
    // Recent-feature smoke tests: stdlib containers, compile-time metaprograms, empty views, the virtual clock,
    // and clean failure of native-only features.
    const features = [
      ['#import "Basic"; #import "Hash_Table"; main :: () { t: Table(int, string); table_set(*t, 1, "one"); ok, v := table_find(*t, 1); print("% %\\n", v, ok); }', "one true\n"],
      ['#import "Basic"; #import "Compiler"; #run { w := compiler_create_workspace("w"); opts := get_build_options(w); opts.output_type = .NO_OUTPUT; set_build_options(opts, w); compiler_begin_intercept(w); add_build_string("main :: () {}", w); while true { m := compiler_wait_for_message(); if m.kind == .COMPLETE break; } compiler_end_intercept(w); print("meta ok\\n"); } main :: () {}', "meta ok\n"],
      ['#import "Basic"; main :: () { a := NewArray(0, s64); if a.data print("bad\\n"); else print("null\\n"); }', "null\n"],
      ['#import "Basic"; main :: () { t := current_time_monotonic(); print("time\\n"); }', "time\n"],
    ];
    let featureId = 10;
    for (const [source, stdout] of features) {
      const result = await run(restarted, ++featureId, source, { fuel: 1000000 });
      assert.deepEqual(result.diagnostics, [], source);
      assert.equal(result.stdout, stdout, source);
    }
    const foreign = await run(restarted, 20, 'puts :: (s: *u8) -> s32 #foreign libc; libc :: #library "libc"; main :: () { puts("x"); }', { fuel: 1000000 });
    assert.equal(foreign.exitCode, null); assert(foreign.diagnostics.length > 0, "Native-only #foreign must fail with a diagnostic, not a crash");
    // Every tests/stdlib program runs in a fresh engine; the pass set must equal tools/playground_stdlib_expected.json.
    const sweep = spawnSync(process.execPath, [fileURLToPath(new URL("./check_playground_stdlib.mjs", import.meta.url)), path.resolve(directory)], { encoding: "utf8" });
    assert.equal(sweep.status, 0, `stdlib pass set check failed:\n${sweep.stdout}${sweep.stderr}`);
    return { stdlibPassSet: sweep.stdout.trim(), actualWorker: true, immutableVfsSnapshot: true, actualLanguageWorker: true, executionCancellation: true, freshWorkerRestart: true };
  } finally { await Promise.all([...workers].map(worker => worker.terminate())); }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.length !== 3) throw new Error("usage: node tools/check_playground_worker.mjs <actual-staged-directory>");
  console.log(JSON.stringify(await checkWorkers(process.argv[2])));
}
