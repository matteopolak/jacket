#!/usr/bin/env node
// Execute the actual wasm module; a native-only test cannot satisfy this gate.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";
import { checkSourceExamples, workspaceFiles } from "./examples_wasm.mjs";
const path = process.argv[2];
if (!path) throw new Error("usage: node tools/check_scripting_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(path));
const fixtures = [
  ["integer entry", { "main.jai": "main :: () -> int { return 42; }" }, 42],
  ["negative result", { "main.jai": "main :: () -> int { return -7; }" }, -7],
  ["runtime phase and compile-time source run", { "main.jai": "seed :: #run answer(); answer :: () -> int { if #compile_time return 40; return 900; } main :: () -> int { if #compile_time return 700; return seed + 2; }" }, 42],
  ["wasm target", { "main.jai": "main :: () -> int { if OS == .WASM return 42; return 1; }" }, 42],
  ["source bundle", { "main.jai": '#load "helper.jai"; main :: () -> int { return answer; }', "helper.jai": "answer :: 42;" }, 42],
  ["nested source import", { "main.jai": '#load "lib/one.jai"; main :: () -> int { return answer; }', "lib/one.jai": '#load "../two.jai"; answer :: result;', "two.jai": "result :: 42;" }, 42],
  ["isolated globals", { "main.jai": "counter: int = 41; main :: () -> int { counter += 1; return counter; }" }, 42],
  ["void entry", { "main.jai": "main :: () {}" }, 0],
  ["standard library output", { "main.jai": '#import "Basic"; main :: () { print("hi %\\n", 42); }' }, 0, "hi 42\n"],
];
for (const [name, files, expected, stdout] of fixtures) {
  for (const label of [name, `${name}, repeat run`]) {
    const result = engine.play(files, "main.jai");
    assert.deepEqual(result.diagnostics, [], label);
    assert.equal(result.exitCode, expected, label);
    if (stdout !== undefined) assert.equal(result.stdout, stdout, label);
  }
}
const missing = engine.play({ "main.jai": "main :: () -> int { return missing; }" }, "main.jai");
assert.equal(missing.exitCode, null);
assert(missing.diagnostics.some(item => /missing/.test(item.message)), "Unknown names are reported as diagnostics");
// The example workspaces (examples/tour) run within the playground's budget.
const examples = await checkSourceExamples(engine);
console.log(`PASS: real WebAssembly compiler (${fixtures.length} fixtures, repeated runs and diagnostics; examples: ${examples.join(", ")})`);

// Arguments and standard input: fixed (`stdin` option) and, with JSPI, supplied by the page as the
// program reads (`jai_stdin_read` waits for a line, answers 0 at the end of input).
const echo = {
  "main.jai": '#import "Basic"; #import "POSIX"; main :: () { args := get_command_line_arguments(); print("argc=% last=%\\n", args.count, args[args.count - 1]); buf: [64] u8; line := fgets(buf.data, 64, stdin); print("got %", to_string(buf.data)); line = fgets(buf.data, 64, stdin); print("then %\\n", line == null); }',
};
const fixed = engine.play(echo, "main.jai", { args: ["main", "a b"], stdin: "hello\n" });
assert.equal(fixed.stdout, "argc=2 last=a b\ngot hello\nthen true\n", fixed.rendered);
if (engine.jspi) {
  const lines = ["typed\n", ""];
  const waited = [];
  const live = await createEngine(await readFile(path), { host: { functions: {
    jai_stdin_read([pointer, capacity], memory) {
      const line = new TextEncoder().encode(lines.shift() ?? "");
      waited.push(line.length);
      return new Promise(resolve => setTimeout(() => {
        new Uint8Array(memory.buffer, Number(pointer), Number(capacity)).set(line);
        resolve(line.length);
      }, 5));
    },
  } } });
  const interactive = await live.playAsync(echo, "main.jai", { args: ["main"] });
  assert.equal(interactive.stdout, "argc=1 last=main\ngot typed\nthen true\n", interactive.rendered);
  assert.deepEqual(waited, [6, 0]);
}
console.log("PASS: arguments and standard input");

// `exit(n)` ends the program with exit code n and keeps the output so far, as returning from main does.
const exited = engine.play({ "main.jai": '#import "Basic"; main :: () { print("before\\n"); exit(7); print("after\\n"); }' }, "main.jai");
assert.equal(exited.exitCode, 7, exited.rendered);
assert.equal(exited.stdout, "before\n");
assert.deepEqual(exited.diagnostics, []);
console.log("PASS: exit");

// The tour runs every stop with no arguments and no input.
const tour = await workspaceFiles(fileURLToPath(new URL("../examples/tour", import.meta.url)));
const tourRun = engine.play(tour, "main.jai", { args: ["main"], budget: 200000000 });
assert.equal(tourRun.exitCode, 0, tourRun.rendered);
assert(tourRun.stdout.includes(" 1. Basics") && tourRun.stdout.includes("That's the tour."), tourRun.stdout);
console.log("PASS: tour");

// The language server in the same module: metaprogram expansions, inlay hints, format strings.
assert(engine.lsp, "the module exports the language server");
let nextId = 0;
const request = (method, params) => {
  const id = ++nextId;
  const reply = engine.lsp({ jsonrpc: "2.0", id, method, params }).find(message => message.id === id);
  assert(reply && !reply.error, `${method}: ${JSON.stringify(reply)}`);
  return reply.result;
};
const legend = request("initialize", { capabilities: { textDocument: { hover: { contentFormat: ["markdown", "plaintext"] } } } }).capabilities.semanticTokensProvider.legend;
assert(legend.tokenTypes.includes("formatSpecifier") && legend.tokenModifiers.includes("macro"));
const uri = "file:///jai-script/main.jai";
const source = [
  '#import "Basic";',
  "LIMIT :: #run 6 * 7;",
  "main :: () {",
  "    count := 3;",
  '    #insert "twice := count * 2;";',
  '    print("% and %\\n", count, twice);',
  "}",
  "",
].join("\n");
engine.lsp({ jsonrpc: "2.0", method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "jai", version: 1, text: source } } });
// Too few arguments is the compiler's warning.
const published = engine.lsp({ jsonrpc: "2.0", method: "textDocument/didOpen", params: { textDocument: { uri: "file:///jai-script/bad.jai", languageId: "jai", version: 1, text: '#import "Basic";\nmain :: () { print("% %\\n", 1); }\n' } } });
assert(published.some(m => m.params?.diagnostics?.some(d => d.message.includes("incorrect number of arguments"))), "format argument mismatch is diagnosed");
const textDocument = { uri };
const hints = request("textDocument/inlayHint", { textDocument, range: { start: { line: 0, character: 0 }, end: { line: 7, character: 0 } } });
assert(hints.some(h => h.label === ": s64" && h.kind === 1), JSON.stringify(hints));
const expansion = request("jai/expansion", { textDocument, position: { line: 4, character: 6 } });
assert(expansion.uri.startsWith("jai-expansion:") && expansion.text.includes("twice := count * 2;"), JSON.stringify(expansion));
assert.equal(request("jai/source", { uri: expansion.uri }), expansion.text);
const actions = request("textDocument/codeAction", { textDocument, range: { start: { line: 4, character: 6 }, end: { line: 4, character: 6 } }, context: { diagnostics: [] } });
assert(actions.some(a => a.title === "Inline #insert" && a.edit.changes[uri][0].newText === "twice := count * 2;"), JSON.stringify(actions));
const hover = request("textDocument/hover", { textDocument, position: { line: 5, character: 12 } });
assert(hover.contents.kind === "markdown" && hover.contents.value.includes("`count: s64`") && hover.contents.value.includes("`twice: s64`"), JSON.stringify(hover));
const runHover = request("textDocument/hover", { textDocument, position: { line: 1, character: 10 } });
assert(runHover.contents.value.startsWith("```jai\n#run = 42: s64\n```"), JSON.stringify(runHover));
const tokens = request("textDocument/semanticTokens/full", { textDocument }).data;
const kinds = new Set(); for (let i = 3; i < tokens.length; i += 5) kinds.add(legend.tokenTypes[tokens[i]]);
assert(kinds.has("formatSpecifier"), [...kinds].join(","));
const stdlibBasic = "file:///stdlib/Basic/module.jai";
assert.deepEqual(request("textDocument/definition", { textDocument, position: { line: 0, character: 10 } }).map(l => l.uri), [stdlibBasic]);
assert(request("textDocument/documentLink", { textDocument }).some(l => l.target === stdlibBasic), "#import strings are links");
assert(typeof request("jai/source", { uri: stdlibBasic }) === "string", "bundled stdlib files are readable");
console.log("PASS: real WebAssembly language server (expansions, inlay hints, code actions, format strings, import links)");
