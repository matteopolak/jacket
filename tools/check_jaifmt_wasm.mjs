// Checks jaifmt.wasm (jaifmt/wasm.jai compiled by `jaic build -os wasm`) the way a page
// uses it: source on stdin, the jaifmt.toml text in --config, the result on stdout. Every
// Jai_Format golden case must format to its .out.jai, and, given a native jaifmt, to exactly
// what `jaifmt --stdin --config <file>` prints. Also checks a config error, refused input and
// a file of several hundred lines, and prints how long the runs take. Usage:
//   node --no-warnings tools/check_jaifmt_wasm.mjs <jaifmt.wasm> [native-jaifmt]
// Needs node 24 (Memory64). See docs/tools/jaifmt.md.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { closeSync, mkdtempSync, openSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { WASI } from "node:wasi";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const [wasmPath, nativePath] = process.argv.slice(2);
if (!wasmPath) throw new Error("usage: node tools/check_jaifmt_wasm.mjs <jaifmt.wasm> [native-jaifmt]");
const module = await WebAssembly.compile(readFileSync(wasmPath));
const scratch = mkdtempSync(path.join(tmpdir(), "jaifmt-wasm-"));

// One run of the module: a fresh instance, stdin and stdout through scratch files.
async function format(source, { config, name = "main.jai" } = {}) {
  const files = ["in", "out", "err"].map(f => path.join(scratch, f));
  writeFileSync(files[0], source);
  const [stdin, stdout, stderr] = [openSync(files[0], "r"), openSync(files[1], "w"), openSync(files[2], "w")];
  const args = ["jaifmt.wasm", "--name", name, ...(config === undefined ? [] : ["--config", config])];
  const wasi = new WASI({ version: "preview1", args, env: {}, stdin, stdout, stderr, returnOnExit: true });
  const started = performance.now();
  const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
  const exitCode = wasi.start(instance);
  const ms = performance.now() - started;
  [stdin, stdout, stderr].forEach(closeSync);
  return { exitCode, stdout: readFileSync(files[1], "utf8"), stderr: readFileSync(files[2], "utf8"), ms };
}

function native(source, config) {
  const args = ["--stdin"];
  if (config !== undefined) {
    writeFileSync(path.join(scratch, "jaifmt.toml"), config);
    args.push("--config", path.join(scratch, "jaifmt.toml"));
  }
  return execFileSync(nativePath, args, { input: source, encoding: "utf8" });
}

const cases = path.join(root, "stdlib/Jai_Format/tests/cases");
const names = readdirSync(cases).filter(n => n.endsWith(".in.jai")).map(n => n.slice(0, -".in.jai".length)).sort();
const times = [];
for (const name of names) {
  const input = readFileSync(path.join(cases, `${name}.in.jai`), "utf8");
  const expected = readFileSync(path.join(cases, `${name}.out.jai`), "utf8");
  let config;
  try {
    config = readFileSync(path.join(cases, `${name}.toml`), "utf8");
  } catch {}
  const result = await format(input, { config });
  assert.equal(result.exitCode, 0, `${name}: ${result.stderr}`);
  assert.equal(result.stdout, expected, `${name}: formatted text differs from ${name}.out.jai`);
  if (nativePath) assert.equal(result.stdout, native(input, config), `${name}: differs from native jaifmt`);
  times.push(result.ms);
}

const badConfig = await format("main :: () {}\n", { config: "indent_width = wide\n" });
assert.equal(badConfig.exitCode, 1);
assert.equal(badConfig.stdout, "");
assert.match(badConfig.stderr, /^jaifmt: jaifmt\.toml: line 1: indent_width/);

const refused = await format("f :: () { x := (1; }\n", { name: "broken.jai" });
assert.equal(refused.exitCode, 1);
assert.equal(refused.stdout, "");
assert.match(refused.stderr, /^jaifmt: broken\.jai:.*unbalanced/);

const large = readFileSync(path.join(root, "jaifmt/main.jai"), "utf8");
const big = await format(large);
assert.equal(big.exitCode, 0, big.stderr);
assert.equal(big.stdout, large, "jaifmt/main.jai is already formatted");
const again = await format(big.stdout);
assert.equal(again.stdout, large, "formatting is idempotent");

rmSync(scratch, { recursive: true, force: true });
const median = times.sort((a, b) => a - b)[times.length >> 1];
console.log(`jaifmt.wasm: ${names.length} golden cases ok${nativePath ? ", identical to native jaifmt" : ""} ` +
  `(median ${median.toFixed(1)} ms), ${large.split("\n").length - 1}-line file in ${Math.min(big.ms, again.ms).toFixed(1)} ms`);
