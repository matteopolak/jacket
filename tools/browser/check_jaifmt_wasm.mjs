// Checks the formatter in the two WebAssembly forms it ships in, with one set of checks:
// every Jai_Format golden case must format to its .out.jai, and a config error, refused input,
// a file of several hundred lines and idempotence must behave. Prints how long the runs take.
//
//   node tools/browser/check_jaifmt_wasm.mjs engine [jai_wasm.wasm | staged-dir]
//       the playground's Format button: jaifmt/playground.jai runs in the wasm interpreter on
//       files in the virtual /workspace. A .wasm path runs with crates/jai-wasm/js/engine.mjs; a
//       staged directory with its own engine (default target/scripting-runtime).
//   node --no-warnings tools/browser/check_jaifmt_wasm.mjs wasi <jaifmt.wasm> [native-jaifmt]
//       jaifmt.wasm (jaifmt/wasm.jai compiled by `jaic -os wasm`) the way a page uses it: source
//       on stdin, the jaifmt.toml text in --config, the result on stdout. Given a native jaifmt,
//       every golden case must also match `jaifmt --stdin --config <file>`. Needs node 24 (Memory64).
//
// See docs/tools/jaifmt.md.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { closeSync, mkdtempSync, openSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { WASI } from "node:wasi";
import { goldenCaseNames, optionalConfig, readUtf8 } from "./formatter_fixture_inputs.mjs";
import { outputDir, root } from "../repo_output.mjs";

const USAGE = "usage: node tools/browser/check_jaifmt_wasm.mjs engine [jai_wasm.wasm | staged-dir]\n" +
  "       node tools/browser/check_jaifmt_wasm.mjs wasi <jaifmt.wasm> [native-jaifmt]";

// A backend runs the formatter once: format(source, { config, name }) resolves to
// { exitCode, stdout, stderr, ms }. The patterns are what that backend prints for each refusal.
async function engineBackend(argument) {
  const target = path.resolve(argument ?? outputDir("scripting-runtime"));
  const wasm = target.endsWith(".wasm") ? target : path.join(target, "jai_wasm.wasm");
  const engineDirectory = target.endsWith(".wasm") ? path.join(root, "crates/jai-wasm/js") : target;
  const { createEngine } = await import(pathToFileURL(path.join(engineDirectory, "engine.mjs")).href);
  const engine = await createEngine(readFileSync(wasm));
  const driver = readFileSync(path.join(root, "jaifmt/playground.jai"), "utf8");
  return {
    label: "jaifmt in the wasm engine",
    goldenName: name => (name === "switch" ? "src/nested/switch.jai" : "main.jai"),
    configError: /jaifmt\.toml: line 1: indent_width/,
    refusal: /unbalanced/,
    async format(source, { config, name = "main.jai" } = {}) {
      const program = driver.replace(/^TARGET :: ".*";$/m, `TARGET :: ${JSON.stringify(name)};`);
      const files = { "__jaifmt__.jai": program, [name]: source };
      if (config !== undefined) files["jaifmt.toml"] = config;
      const started = performance.now();
      const result = engine.play(files, "__jaifmt__.jai");
      const errors = (result.diagnostics ?? []).filter(d => d.severity === "error");
      assert.equal(errors.length, 0, `driver did not compile: ${JSON.stringify(errors[0])}`);
      return { exitCode: Number(result.exitCode), stdout: result.stdout, stderr: result.stderr, ms: performance.now() - started };
    },
    // The playground's own config (as on the portfolio) and the examples users try first.
    async extra(format) {
      const portfolio = 'indent_width = 4\nmax_blank_lines = 2\nbrace_style = "same_line"\n';
      const messy = 'main :: ()\n{\n    a: int,     b: int;\n    arr := int.[1,2 ,  3];\n    for arr    {\n    }\n' +
        '    x:=1;   if x>0 {print("hi %\\n",x);}\n    if x ==\n    {\n        case 1; a = 1;\n    }\n}\n';
      const tidy = 'main :: () {\n    a: int, b: int;\n    arr := int.[1, 2, 3];\n    for arr {}\n    x := 1;\n' +
        '    if x > 0 {\n        print("hi %\\n", x);\n    }\n    if x == {\n        case 1; a = 1;\n    }\n}\n';
      const user = await format(messy, { config: portfolio });
      assert.equal(user.exitCode, 0, user.stderr);
      assert.equal(user.stdout, tidy);
      assert.equal((await format(tidy, { config: portfolio })).stdout, tidy, "formatting is idempotent");
    },
    close() {},
  };
}

async function wasiBackend(wasmPath, nativePath) {
  if (!wasmPath) throw new Error(USAGE);
  const module = await WebAssembly.compile(readFileSync(wasmPath));
  const scratch = mkdtempSync(path.join(tmpdir(), "jaifmt-wasm-"));
  return {
    label: "jaifmt.wasm",
    nativeNote: nativePath ? ", identical to native jaifmt" : "",
    goldenName: () => "main.jai",
    configError: /^jaifmt: jaifmt\.toml: line 1: indent_width/,
    refusal: /^jaifmt: broken\.jai:.*unbalanced/,
    // One run of the module: a fresh instance, stdin and stdout through scratch files.
    async format(source, { config, name = "main.jai" } = {}) {
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
      return { exitCode, stdout: readUtf8(files[1]), stderr: readUtf8(files[2]), ms };
    },
    // What the native formatter prints for the same input, when one was supplied.
    native: nativePath && ((source, config) => {
      const args = ["--stdin"];
      if (config !== undefined) {
        writeFileSync(path.join(scratch, "case.toml"), config);
        args.push("--config", path.join(scratch, "case.toml"));
      }
      // From scratch, so a config-less case does not pick up the repository's own jaifmt.toml.
      return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(
        execFileSync(path.resolve(nativePath), args, { input: source, cwd: scratch }));
    }),
    close() { rmSync(scratch, { recursive: true, force: true }); },
  };
}

const [mode, ...rest] = process.argv.slice(2);
const backends = { engine: engineBackend, wasi: wasiBackend };
if (!backends[mode]) throw new Error(USAGE);
const backend = await backends[mode](...rest);
const format = (source, options) => backend.format(source, options);

const cases = path.join(root, "stdlib/Extensions/Jai_Format/tests/fixtures");
const names = goldenCaseNames(cases);
assert.ok(names.length > 0, "no formatter golden cases found");
const times = [];
for (const name of names) {
  const input = readUtf8(path.join(cases, `${name}.in.jai`));
  const expected = readUtf8(path.join(cases, `${name}.out.jai`));
  const config = optionalConfig(path.join(cases, `${name}.toml`));
  const result = await format(input, { config, name: backend.goldenName(name) });
  assert.equal(result.exitCode, 0, `${name}: ${result.stderr}`);
  assert.equal(result.stdout, expected, `${name}: formatted text differs from ${name}.out.jai`);
  if (backend.native) assert.equal(result.stdout, backend.native(input, config), `${name}: differs from native jaifmt`);
  times.push(result.ms);
}

await backend.extra?.(format);

const badConfig = await format("main :: () {}\n", { config: "indent_width = wide\n" });
assert.equal(badConfig.exitCode, 1);
assert.equal(badConfig.stdout, "");
assert.match(badConfig.stderr, backend.configError);

const refused = await format("f :: () { x := (1; }\n", { name: "broken.jai" });
assert.equal(refused.exitCode, 1);
assert.equal(refused.stdout, "");
assert.match(refused.stderr, backend.refusal);

const large = readUtf8(path.join(root, "jaifmt/main.jai"));
const big = await format(large);
assert.equal(big.exitCode, 0, big.stderr);
assert.equal(big.stdout, large, "jaifmt/main.jai is already formatted");
const again = await format(big.stdout);
assert.equal(again.stdout, large, "formatting is idempotent");

backend.close();
const median = times.sort((a, b) => a - b)[times.length >> 1];
console.log(`${backend.label}: ${names.length} golden cases ok${backend.nativeNote ?? ""} ` +
  `(median ${median.toFixed(1)} ms), ${large.split("\n").length - 1}-line file in ${Math.min(big.ms, again.ms).toFixed(1)} ms`);
