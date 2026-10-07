#!/usr/bin/env node
// Checks a staged browser bundle and executes its own engine.mjs and Wasm, never a source-tree copy.
// The bundle is the compiler module plus embedder glue and the tour workspace; there is no UI
// (docs/browser/playground.md).
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, readdir, lstat, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { checkExample, exampleCases } from "./examples_wasm.mjs";

export const BUNDLE_FILES = ["README.md", "build-metadata.json", "engine.mjs", "jai_wasm.wasm", "jaifmt-playground.jai", "jaifmt.wasm", "tour.json",
  "webgpu_bindings.generated.mjs", "webgpu_host.mjs"];
/** Example workspaces shipped as folders, each described by `<name>.json` (tools/build_scripting_wasm.py). */
export const BUNDLE_EXAMPLES = ["tour"];

/** Relative paths of every regular file under `directory`; symlinks and other file types are rejected. */
async function regularFiles(directory, prefix = "") {
  const found = [];
  for (const name of (await readdir(path.join(directory, prefix))).sort()) {
    const relative = prefix ? `${prefix}/${name}` : name;
    const information = await lstat(path.join(directory, relative));
    assert(!information.isSymbolicLink(), "Release assets cannot be symlinks");
    if (information.isDirectory()) found.push(...await regularFiles(directory, relative));
    else {
      assert(information.isFile(), `Release assets must be regular files: ${relative}`);
      found.push(relative);
    }
  }
  return found;
}

/** The files of a staged example workspace, checked against its index. */
export async function exampleFiles(directory, name) {
  const index = JSON.parse(await readFile(path.join(directory, `${name}.json`), "utf8"));
  assert.equal(index.schema_version, 1);
  const listed = (await regularFiles(directory, name)).map(file => file.slice(name.length + 1));
  assert.deepEqual([...index.files].sort(), listed, `${name}.json must list exactly the files under ${name}/`);
  assert(index.files.includes(index.main), `${name}.json names a main file it does not list`);
  const files = {};
  for (const file of listed) files[file] = await readFile(path.join(directory, name, file), "utf8");
  return { main: index.main, files };
}

export async function inspectAssets(directory) {
  const entries = await regularFiles(directory);
  const nested = entries.filter(name => name.includes("/"));
  assert.deepEqual(entries.filter(name => !name.includes("/")), BUNDLE_FILES, "The bundle holds exactly the Wasm module, its glue, the formatter driver, metadata, README and the tour index");
  for (const name of nested) assert(BUNDLE_EXAMPLES.includes(name.split("/")[0]), `Unexpected bundle folder: ${name}`);
  for (const name of BUNDLE_EXAMPLES) await exampleFiles(directory, name);
  const metadata = JSON.parse(await readFile(path.join(directory, "build-metadata.json"), "utf8"));
  assert.equal(metadata.schema_version, 1);
  assert.match(metadata.commit, /^[0-9a-f]{40}$/);
  const digest = createHash("sha256").update(await readFile(path.join(directory, "jai_wasm.wasm"))).digest("hex");
  assert.equal(metadata.wasm_sha256, digest, "build-metadata.json must describe the staged module");
  const formatter = createHash("sha256").update(await readFile(path.join(directory, "jaifmt.wasm"))).digest("hex");
  assert.equal(metadata.jaifmt_wasm_sha256, formatter, "build-metadata.json must describe the staged jaifmt.wasm");
  const engine = await readFile(path.join(directory, "engine.mjs"), "utf8");
  assert(!/^\s*(?:import\b|export\b[^;\n]*\bfrom\b)|\bimport\s*\(/m.test(engine), "engine.mjs must be self-contained");
  // The WebGPU host (docs/stdlib/webgpu.md) imports only its generated bindings, which import nothing.
  const imports = text => [...text.matchAll(/^\s*import\b[^;]*?from\s*"([^"]+)"|\bimport\s*\(/gm)].map(m => m[1] ?? "dynamic import");
  assert.deepEqual(imports(await readFile(path.join(directory, "webgpu_host.mjs"), "utf8")), ["./webgpu_bindings.generated.mjs"],
    "webgpu_host.mjs may import only ./webgpu_bindings.generated.mjs");
  assert.deepEqual(imports(await readFile(path.join(directory, "webgpu_bindings.generated.mjs"), "utf8")), [],
    "webgpu_bindings.generated.mjs must be self-contained");
  return metadata;
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

/**
 * Formats `source` with jaifmt.wasm in a child node process (WASI preview 1, Memory64: node 24).
 * The module goes by path: Linux caps one argument at 128 KiB, so its bytes cannot.
 */
function formatWithWasi(wasmPath, source) {
  const script = `
    import { readFileSync } from "node:fs";
    import { WASI } from "node:wasi";
    const bytes = readFileSync(process.argv[1]);
    const wasi = new WASI({ version: "preview1", args: ["jaifmt.wasm"], env: {}, returnOnExit: true });
    const instance = await WebAssembly.instantiate(await WebAssembly.compile(bytes), wasi.getImportObject());
    process.exitCode = wasi.start(instance);`;
  const run = spawnSync(process.execPath, ["--no-warnings", "--input-type=module", "-e", script, path.resolve(wasmPath)], { input: source, encoding: "utf8" });
  if (run.error) throw run.error;
  assert.equal(run.status, 0, `jaifmt.wasm failed: ${run.stderr}`);
  return run.stdout;
}

export async function checkRelease(directory, { stdlib = true } = {}) {
  const metadata = await inspectAssets(directory);
  const { createEngine } = await import(pathToFileURL(path.join(directory, "engine.mjs")).href);
  assert.equal(typeof createEngine, "function");
  const engine = await createEngine(await readFile(path.join(directory, "jai_wasm.wasm")));
  const play = (source, files = {}, options) => engine.play({ ...files, "main.jai": source }, "main.jai", options);
  assert.equal(play("main :: () -> int { return 42; }").exitCode, 42);
  assert.equal(play("main :: () -> int { if OS == .WASM return 42; return 1; }").exitCode, 42, "Browser builds target WASM");
  assert.equal(play('seed :: #run answer(); answer :: () -> int { if #compile_time return 40; return 900; } main :: () -> int { if #compile_time return 700; return seed + 2; }').exitCode, 42);
  assert.equal(play('#load "nested/helper.jai"; main :: () -> int { return answer; }', { "nested/helper.jai": "answer :: 42;" }).exitCode, 42);
  assert.equal(play('#load "lib/one.jai"; main :: () -> int { return answer; }', { "lib/one.jai": '#load "../two.jai"; answer :: result;', "two.jai": "result :: 42;" }).exitCode, 42, "Nested parent paths stay inside the virtual root");
  const missing = play("main :: () -> int { return missing; }");
  assert.equal(missing.exitCode, null);
  assert(missing.diagnostics.some(item => /missing/.test(item.message)));
  // The staged WebGPU host loads and builds its table of host functions; without a GPU, a
  // program learns that WebGPU is unavailable instead of failing.
  const { createWebGPUHost } = await import(pathToFileURL(path.join(directory, "webgpu_host.mjs")).href);
  const gpuless = createWebGPUHost({ gpu: undefined });
  assert.equal(typeof gpuless.functions.wgpuDeviceCreateBuffer, "function");
  const hosted = await createEngine(await readFile(path.join(directory, "jai_wasm.wasm")), { host: gpuless });
  const webgpu = '#import "Basic"; #import "WebGPU"; main :: () { print("%\\n", webgpu_available()); }';
  assert.equal(hosted.play({ "main.jai": webgpu }, "main.jai").stdout, "false\n", "WebGPU without a GPU");
  assert.equal(play(webgpu).stdout, "false\n", "WebGPU without a host");
  // Recent-feature smoke tests: stdlib containers, compile-time metaprograms, empty views, the virtual clock.
  const features = [
    ['#import "Basic"; #import "Hash_Table"; main :: () { t: Table(int, string); table_set(*t, 1, "one"); ok, v := table_find(*t, 1); print("% %\\n", v, ok); }', "one true\n"],
    ['#import "Basic"; #import "Compiler"; #run { w := compiler_create_workspace("w"); opts := get_build_options(w); opts.output_type = .NO_OUTPUT; set_build_options(opts, w); compiler_begin_intercept(w); add_build_string("main :: () {}", w); while true { m := compiler_wait_for_message(); if m.kind == .COMPLETE break; } compiler_end_intercept(w); print("meta ok\\n"); } main :: () {}', "meta ok\n"],
    ['#import "Basic"; main :: () { a := NewArray(0, s64); if a.data print("bad\\n"); else print("null\\n"); }', "null\n"],
    ['#import "Basic"; main :: () { t := current_time_monotonic(); print("time\\n"); }', "time\n"],
  ];
  for (const [source, stdout] of features) {
    const result = play(source);
    assert.deepEqual(result.diagnostics, [], source);
    assert.equal(result.stdout, stdout, source);
  }
  // Embedders get stdout/stderr separately and in write order, and can bound runaway programs.
  const streams = play('#import "Basic"; main :: () { print("a\\n"); log_error("b"); print("c\\n"); }');
  assert.equal(streams.stdout, "a\nc\n"); assert.equal(streams.stderr, "b\n");
  assert.deepEqual(streams.output, [{ stream: "stdout", text: "a\n" }, { stream: "stderr", text: "b\n" }, { stream: "stdout", text: "c\n" }]);
  const bounded = play('#import "Basic"; main :: () { print("go\\n"); while true {} }', {}, { budget: 200000 });
  assert.equal(bounded.stdout, "go\n"); assert.equal(bounded.exitCode, null);
  assert(bounded.diagnostics.some(item => /execution budget exhausted/.test(item.message)), JSON.stringify(bounded.diagnostics));
  const foreign = play('puts :: (s: *u8) -> s32 #foreign libc; libc :: #library "libc"; main :: () { puts("x"); }');
  assert.equal(foreign.exitCode, null); assert(foreign.diagnostics.length > 0, "Native-only #foreign must fail with a diagnostic, not a crash");
  // The staged formatter driver formats /workspace/main.jai and prints the result.
  const driver = await readFile(path.join(directory, "jaifmt-playground.jai"), "utf8");
  const formatted = engine.play({ "__jaifmt__.jai": driver, "main.jai": "main::(){\nx:=1;\n}\n" }, "__jaifmt__.jai");
  assert.equal(formatted.exitCode, 0, formatted.stderr);
  assert.match(formatted.stdout, /^main :: \(\) \{\n\s+x := 1;\n\}\n$/);
  // So does jaifmt.wasm, the formatter compiled to WebAssembly, reading stdin under WASI.
  assert.equal(formatWithWasi(path.join(directory, "jaifmt.wasm"), "main::(){\nx:=1;\n}\n"), formatted.stdout);
  // The staged tour runs as the playground opens it: its own files, under the playground's budget.
  const tours = [];
  for (const testCase of (await exampleCases()).filter(item => item.bundle)) {
    const { main, files } = await exampleFiles(directory, testCase.bundle);
    assert.equal(main, testCase.main);
    tours.push(`${testCase.bundle} ${Math.round(checkExample(engine, testCase, files))} ms`);
  }
  const lsp = typeof engine.lsp === "function";
  if (lsp) {
    checkLanguageServer(engine);
    assert.equal(play("main :: () -> int { return 42; }").exitCode, 42, "LSP document state must not replace runtime source");
  }
  let stdlibPassSet;
  if (stdlib) {
    // Every tests/stdlib program runs in a fresh engine and must pass.
    const sweep = spawnSync(process.execPath, [fileURLToPath(new URL("./check_playground_stdlib.mjs", import.meta.url)), path.resolve(directory)], { encoding: "utf8" });
    assert.equal(sweep.status, 0, `stdlib tests failed in the browser engine:\n${sweep.stdout}${sweep.stderr}`);
    stdlibPassSet = sweep.stdout.trim();
  }
  return { commit: metadata.commit, runtime: true, lsp, tours, stdlibPassSet };
}

if (process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url))) {
  if (!(process.argv.length === 3 || (process.argv.length === 5 && process.argv[3] === "--report"))) throw new Error("usage: node tools/check_browser_release.mjs <staged-directory> [--report <json-path>]");
  const result = await checkRelease(path.resolve(process.argv[2]));
  if (process.argv[3] === "--report") await writeFile(process.argv[4], JSON.stringify(result) + "\n");
  console.log(`PASS: staged browser bundle ${result.commit}, runtime=true, lsp=${result.lsp}; ${result.tours.join(", ")}; ${result.stdlibPassSet}`);
}
