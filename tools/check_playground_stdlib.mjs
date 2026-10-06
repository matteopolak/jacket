// Runs every tests/stdlib/*.jai through the browser engine (a fresh wasm instance per test); every one must pass.
// Usage:
//   node tools/check_playground_stdlib.mjs <staged-dir> [name.jai...]
// A test passes when it finishes with exit code 0 and no error diagnostics, the same bar as the native loop
// (`jaic run` exits 0). There is no exclusion list: a test that needs something the browser lacks (processes,
// native libraries, a window system) says so itself with `OS == .WASM`, the target the engine compiles for.
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const testDir = path.join(root, "tests/stdlib");

async function collectModules(dir, base, files) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) await collectModules(full, base, files);
    else if (entry.isFile() && /\.(jai|txt|json|md)$/.test(entry.name)) files[path.relative(base, full).replaceAll("\\", "/")] = await readFile(full, "utf8");
  }
}

if (!isMainThread) {
  const { directory, name } = workerData;
  const { createEngine } = await import(pathToFileURL(path.join(directory, "engine.mjs")).href);
  const files = {};
  await collectModules(testDir, testDir, files);
  const engine = await createEngine(await readFile(path.join(directory, "jai_wasm.wasm")));
  parentPort.postMessage(engine.play(files, name));
} else {
  const argv = process.argv.slice(2);
  const positional = argv;
  if (!positional[0] || argv.some(a => a.startsWith("--"))) throw new Error("usage: node tools/check_playground_stdlib.mjs <staged-dir> [test.jai...]");
  const directory = path.resolve(positional[0]);
  const only = positional.slice(1);
  const names = (await readdir(testDir)).filter(n => n.endsWith(".jai")).sort().filter(n => !only.length || only.includes(n));
  const timeoutMs = 120000;
  const runOne = name => new Promise(resolve => {
    const worker = new Worker(fileURLToPath(import.meta.url), { workerData: { directory, name }, resourceLimits: { maxOldGenerationSizeMb: 4096 } });
    const timer = setTimeout(() => { worker.terminate(); resolve({ pass: false, why: "timeout" }); }, timeoutMs);
    const done = value => { clearTimeout(timer); resolve(value); };
    worker.on("message", r => {
      const errors = (r.diagnostics ?? []).filter(d => d.severity === "error");
      const pass = r.exitCode === 0 && errors.length === 0;
      const first = errors[0] ? `${errors[0].file}:${errors[0].line}: ${errors[0].message}` : (r.stderr || "").split("\n")[0];
      if (process.env.PLAYGROUND_VERBOSE) console.log(`--- ${name}\n${r.stdout}\n${r.stderr}\n${r.rendered}`);
      done({ pass, why: pass ? "" : `exit ${r.exitCode}; ${first}` });
    });
    worker.on("error", e => done({ pass: false, why: `crash: ${String(e.message).split("\n")[0]}` }));
    worker.on("exit", () => done({ pass: false, why: "worker exited" }));
  });
  const results = new Map();
  let next = 0;
  await Promise.all(Array.from({ length: 4 }, async () => { while (next < names.length) { const n = names[next++]; results.set(n, await runOne(n)); } }));
  const passed = names.filter(n => results.get(n).pass);
  const failed = names.filter(n => !results.get(n).pass);
  for (const n of failed) console.log(`FAIL ${n}: ${results.get(n).why}`);
  const missing = only.filter(n => !names.includes(n));
  if (missing.length) console.log(`no such test in tests/stdlib: ${missing.join(", ")}`);
  console.log(`${passed.length} of ${names.length} stdlib tests pass in the playground`);
  // A failure blocks CI and the browser release (check_browser_release.mjs), so a broken bundle is never published.
  if (failed.length || missing.length || !names.length) process.exit(1);
}
