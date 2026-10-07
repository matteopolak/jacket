// Formats Jai through the browser engine the way a playground Format button does: the
// jaifmt/playground.jai driver runs in the wasm interpreter on files in the virtual
// /workspace. Checks every Jai_Format golden case, a config error, refused input and an
// already formatted file of several hundred lines, and prints how long each run takes. Usage:
//   node tools/check_jai_format_wasm.mjs [jai_wasm.wasm | staged-dir]   (default artifacts/scripting-runtime)
// A .wasm path runs with crates/jai-wasm/js/engine.mjs; a staged directory with its own engine.
import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = path.resolve(process.argv[2] ?? path.join(root, "artifacts/scripting-runtime"));
const wasm = target.endsWith(".wasm") ? target : path.join(target, "jai_wasm.wasm");
const engineDirectory = target.endsWith(".wasm") ? path.join(root, "crates/jai-wasm/js") : target;
const { createEngine } = await import(pathToFileURL(path.join(engineDirectory, "engine.mjs")).href);
const engine = await createEngine(await readFile(wasm));
const driver = await readFile(path.join(root, "jaifmt/playground.jai"), "utf8");

function format(source, { config, target = "main.jai" } = {}) {
  const files = { "__jaifmt__.jai": driver.replace(/^TARGET :: ".*";$/m, `TARGET :: ${JSON.stringify(target)};`), [target]: source };
  if (config !== undefined) files["jaifmt.toml"] = config;
  const started = performance.now();
  const result = engine.play(files, "__jaifmt__.jai");
  const errors = (result.diagnostics ?? []).filter(d => d.severity === "error");
  assert.equal(errors.length, 0, `driver did not compile: ${JSON.stringify(errors[0])}`);
  return { ...result, exitCode: Number(result.exitCode), ms: performance.now() - started };
}

const cases = path.join(root, "stdlib/Extensions/Jai_Format/tests/cases");
const names = (await readdir(cases)).filter(n => n.endsWith(".in.jai")).map(n => n.slice(0, -".in.jai".length)).sort();
const times = [];
for (const name of names) {
  const input = await readFile(path.join(cases, `${name}.in.jai`), "utf8");
  const expected = await readFile(path.join(cases, `${name}.out.jai`), "utf8");
  const config = await readFile(path.join(cases, `${name}.toml`), "utf8").catch(() => undefined);
  const result = format(input, { config, target: name === "switch" ? "src/nested/switch.jai" : "main.jai" });
  assert.equal(result.exitCode, 0, `${name}: ${result.stderr}`);
  assert.equal(result.stdout, expected, `${name}: formatted text differs from ${name}.out.jai`);
  times.push(result.ms);
}

// The playground's own config (as on the portfolio) and the examples users try first.
const portfolio = 'indent_width = 4\nmax_blank_lines = 2\nbrace_style = "same_line"\n';
const messy = 'main :: ()\n{\n    a: int,     b: int;\n    arr := int.[1,2 ,  3];\n    for arr    {\n    }\n' +
  '    x:=1;   if x>0 {print("hi %\\n",x);}\n    if x ==\n    {\n        case 1; a = 1;\n    }\n}\n';
const tidy = 'main :: () {\n    a: int, b: int;\n    arr := int.[1, 2, 3];\n    for arr {}\n    x := 1;\n' +
  '    if x > 0 {\n        print("hi %\\n", x);\n    }\n    if x == {\n        case 1; a = 1;\n    }\n}\n';
const user = format(messy, { config: portfolio });
assert.equal(user.exitCode, 0, user.stderr);
assert.equal(user.stdout, tidy);
assert.equal(format(tidy, { config: portfolio }).stdout, tidy, "formatting is idempotent");

const badConfig = format("main :: () {}\n", { config: "indent_width = wide\n" });
assert.equal(badConfig.exitCode, 1);
assert.equal(badConfig.stdout, "");
assert.match(badConfig.stderr, /jaifmt\.toml: line 1: indent_width/);

const refused = format("f :: () { x := (1; }\n");
assert.equal(refused.exitCode, 1);
assert.equal(refused.stdout, "");
assert.match(refused.stderr, /unbalanced/);

const large = await readFile(path.join(root, "jaifmt/main.jai"), "utf8");
const big = format(large);
assert.equal(big.exitCode, 0, big.stderr);
assert.equal(big.stdout, large, "jaifmt/main.jai is already formatted");

const median = times.sort((a, b) => a - b)[times.length >> 1];
console.log(`jaifmt in the wasm engine: ${names.length} golden cases ok (median ${median.toFixed(0)} ms), ` +
  `${large.split("\n").length - 1}-line file in ${big.ms.toFixed(0)} ms`);
