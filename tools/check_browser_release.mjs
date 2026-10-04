#!/usr/bin/env node
import { checkWorkers } from "./check_playground_worker.mjs";
// Check the staged assets and execute their own engine/Wasm, never a source-tree wrapper.
import assert from "node:assert/strict";
import { readFile, readdir, lstat, writeFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";

function relativeDependency(from, dependency, files) {
  if (dependency.startsWith("#") || dependency.startsWith("data:")) return;
  assert(!/^(?:[a-z][a-z\d+.-]*:|\/|\\)/i.test(dependency), `Asset requires a non-relative URL: ${dependency}`);
  const name = path.posix.normalize(path.posix.join(path.posix.dirname(from), dependency.split(/[?#]/)[0]));
  assert(name !== ".." && !name.startsWith("../"), `Asset URL escapes release: ${dependency}`);
  assert(files.has(name), `Missing staged dependency: ${from} -> ${dependency}`);
}

export async function inspectAssets(directory) {
  const files = new Set();
  async function walk(name = "") {
    for (const entry of await readdir(path.join(directory, name))) {
      const relative = name ? `${name}/${entry}` : entry;
      const information = await lstat(path.join(directory, relative));
      assert(!information.isSymbolicLink(), "Release assets cannot be symlinks");
      if (information.isDirectory()) await walk(relative);
      else { assert(information.isFile()); files.add(relative); }
    }
  }
  await walk();
  for (const name of ["index.html", "worker.mjs", "engine.mjs", "jai_wasm.wasm", "release.json"]) assert(files.has(name), `Missing ${name}`);
  const release = JSON.parse(await readFile(path.join(directory, "release.json"), "utf8"));
  assert.equal(release.schema_version, 1);
  assert.match(release.commit, /^[0-9a-f]{40}$/);
  for (const name of files) {
    if (!/\.(?:html|css|m?js)$/.test(name)) continue;
    const text = await readFile(path.join(directory, name), "utf8");
    const patterns = name.endsWith(".html")
      ? [/<(?:script|link)\b[^>]*\b(?:src|href)\s*=\s*["']([^"']+)["']/gi]
      : name.endsWith(".css") ? [/url\(\s*["']?([^\s"')]+)["']?\s*\)/gi]
        : [/(?<![#\w.])\b(?:import|export)\s+(?:[^;\n]*?\sfrom\s*)?["']([^"']+)["']/g,
           /\b(?:import|fetch|Worker|URL)\s*\(\s*["']([^"']+)["']/g];
    for (const pattern of patterns) for (const match of text.matchAll(pattern)) relativeDependency(name, match[1], files);
  }
  return release;
}

function checkLanguageServer(engine) {
  const uri = "file:///workspace/main.jai";
  function request(id, method, params) {
    const response = engine.lsp({ jsonrpc: "2.0", id, method, params }).find(item => item.id === id);
    assert(response && !response.error && response.result != null, `Language request failed: ${method}`);
    return response.result;
  }
  const initialized = request(1, "initialize", { processId: null, rootUri: "file:///workspace/", capabilities: {} });
  assert.equal(initialized.capabilities.definitionProvider, true);
  assert.equal(initialized.capabilities.hoverProvider, true);
  assert(initialized.capabilities.completionProvider);
  engine.lsp({ jsonrpc: "2.0", method: "initialized", params: {} });
  const text = "answer :: () -> int { return 42; }\nmain :: () -> int { return answer(); }";
  engine.lsp({ jsonrpc: "2.0", method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "jai", version: 1, text } } });
  const position = { line: 1, character: text.split("\n")[1].indexOf("answer") + 1 };
  const definition = request(2, "textDocument/definition", { textDocument: { uri }, position });
  assert(Array.isArray(definition) && definition.some(item => item.uri === uri && item.range.start.line === 0));
  const hover = request(3, "textDocument/hover", { textDocument: { uri }, position });
  assert.equal(hover.contents.kind, "plaintext");
  assert.match(hover.contents.value, /answer/);
  const completion = request(4, "textDocument/completion", { textDocument: { uri }, position });
  assert(completion.items.some(item => item.label === "answer"));
  const changed = engine.lsp({ jsonrpc: "2.0", method: "textDocument/didChange", params: { textDocument: { uri, version: 2 }, contentChanges: [{ text: "main :: () -> int { return ;" }] } });
  const diagnostic = changed.find(item => item.method === "textDocument/publishDiagnostics" && item.params.uri === uri && item.params.version === 2);
  assert(diagnostic && diagnostic.params.diagnostics.length > 0, "Changed source must produce current-version syntax diagnostics");
}

export async function checkRelease(directory) {
  const release = await inspectAssets(directory);
  const { createEngine } = await import(pathToFileURL(path.join(directory, "engine.mjs")).href);
  assert.equal(typeof createEngine, "function");
  const engine = await createEngine(await readFile(path.join(directory, "jai_wasm.wasm")));
  const play = (source, files = {}) => engine.play({ ...files, "main.jai": source }, "main.jai");
  assert.equal(play("main :: () -> int { return 42; }").exitCode, 42);
  assert.equal(play("main :: () -> int { if OS == .WASM return 42; return 1; }").exitCode, 42, "Browser builds target WASM");
  assert.equal(play('seed :: #run answer(); answer :: () -> int { if #compile_time return 40; return 900; } main :: () -> int { if #compile_time return 700; return seed + 2; }').exitCode, 42);
  assert.equal(play('#load "nested/helper.jai"; main :: () -> int { return answer; }', { "nested/helper.jai": "answer :: 42;" }).exitCode, 42);
  assert.equal(play('#load "lib/one.jai"; main :: () -> int { return answer; }', { "lib/one.jai": '#load "../two.jai"; answer :: result;', "two.jai": "result :: 42;" }).exitCode, 42, "Nested parent paths stay inside the virtual root");
  const missing = play("main :: () -> int { return missing; }");
  assert.equal(missing.exitCode, null);
  assert(missing.diagnostics.some(item => /missing/.test(item.message)));
  const requiresLanguageServer = await lstat(path.join(directory, "lsp-client.mjs")).then(() => true, error => { if (error.code === "ENOENT") return false; throw error; });
  if (requiresLanguageServer) assert.equal(typeof engine.lsp, "function", "Rich editor requires the real shared language-server Wasm bridge");
  const lsp = typeof engine.lsp === "function";
  if (lsp) {
    checkLanguageServer(engine);
    assert.equal(play("main :: () -> int { return 42; }").exitCode, 42, "LSP document state must not replace runtime source");
  }
  const worker = await checkWorkers(directory);
  return { commit: release.commit, runtime: true, lsp, worker };
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  if (!(process.argv.length === 3 || (process.argv.length === 5 && process.argv[3] === "--report"))) throw new Error("usage: node tools/check_browser_release.mjs <staged-directory> [--report <json-path>]");
  const result = await checkRelease(path.resolve(process.argv[2]));
  if (process.argv[3] === "--report") await writeFile(process.argv[4], JSON.stringify(result) + "\n");
  console.log(`PASS: staged real browser compiler ${result.commit}, runtime=true, lsp=${result.lsp}, source execution, phase, bundle and diagnostics`);
}
