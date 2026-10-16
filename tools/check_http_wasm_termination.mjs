// Exercise the actual browser compiler/SandboxHost; no injected host functions.
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const [binary, output] = process.argv.slice(2);
if (!binary || !output) throw new Error("usage: node tools/check_http_wasm_termination.mjs <jai_wasm.wasm> <results.json>");
const root = fileURLToPath(new URL("../", import.meta.url));
const cases = [
  ["tests/extensions/http-wasm-termination-failure.jai", 1, "FAIL: independent WASM termination control\n1 failure(s)\n"],
  ["tests/extensions/http-wasm-termination-status.jai", 37, "before exit 37\n"],
  ["stdlib/Extensions/Http/tests/codec.jai", 0, "ok\n"],
  ["stdlib/Extensions/Http/tests/smuggling.jai", 0, "ok\n"],
  ["tests/extensions/http-codec-roundtrip.jai", 0, "ok\n"],
];
const files = {};
for (const source of [...cases.map(row => row[0]), "stdlib/Extensions/Http/tests/lib/support.jai"]) {
  files[source] = await readFile(path.join(root, source), "utf8");
}
const engine = await createEngine(await readFile(binary));
const results = [];
// Repeat the failed check after successes: failure counters and exit state must reset.
for (const [source, exit, stdout] of [...cases, cases[0]]) {
  try {
    const result = engine.play(files, source, { budget: 50_000_000 });
    const passed = result.exitCode === exit && result.stdout === stdout && result.stderr === "" && result.diagnostics.length === 0;
    results.push({ source, expectedExit: exit, expectedStdout: stdout, passed, result });
  } catch (error) {
    results.push({ source, expectedExit: exit, passed: false, error: String(error) });
  }
  await writeFile(output, JSON.stringify(results, null, 2) + "\n");
  console.log(`${source}: ${results.at(-1).passed ? "PASS" : "FAIL"}`);
}
if (results.some(row => !row.passed)) process.exitCode = 1;
