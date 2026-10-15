// Browser verification of reviewed contracts and independent regressions.
// Native SDK/graphics cases and explicit unavailable-foreign diagnostics are reported
// separately, never as passing browser execution. No legacy failure baseline is used.
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";
import { readFile, readdir } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const python = process.env.PYTHON || (process.platform === "win32" ? "python" : "python3");
const catalogue = isMainThread ? JSON.parse(execFileSync(python, [path.join(root, "tools/stdlib_test_catalog.py")], { encoding: "utf8" })) : [];

async function collectModules(dir, base, files) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) await collectModules(full, base, files);
    else if (entry.isFile() && /\.(jai|txt|json|md)$/.test(entry.name)) files[path.relative(base, full).replaceAll("\\", "/")] = await readFile(full, "utf8");
  }
}

if (!isMainThread) {
  const { directory, test } = workerData;
  const { createEngine } = await import(pathToFileURL(path.join(directory, "engine.mjs")).href);
  const parts = test.path.split("/");
  const base = path.join(root, ...parts.slice(0, test.family === "modules" ? 1 : 2));
  const files = {};
  // Only our NEW library and independently maintained test family, never corpus vendors.
  await collectModules(base, base, files);
  const engine = await createEngine(await readFile(path.join(directory, "jai_wasm.wasm")));
  parentPort.postMessage(engine.play(files, path.relative(base, path.join(root, test.path)).replaceAll("\\", "/")));
} else {
  const [staged, ...only] = process.argv.slice(2);
  if (!staged || only.some(a => a.startsWith("--"))) throw new Error("usage: node tools/check_playground_stdlib.mjs <staged-dir> [case-id...]");
  const directory = path.resolve(staged);
  const tests = catalogue.filter(c => !only.length || only.includes(c.id));
  for (const test of [...tests]) {
    if (test.positive_baseline) {
      const baseline = catalogue.find(c => c.id === "contracts/" + test.positive_baseline);
      if (!baseline) throw new Error("missing positive baseline for " + test.id);
      if (!tests.includes(baseline)) tests.push(baseline);
    }
  }
  const runOne = test => new Promise(resolve => {
    if (["native", "graphics"].includes(test.family)) {
      resolve({ unavailable: "native SDK/OS/graphics regression requires native CI" });
      return;
    }
    const worker = new Worker(fileURLToPath(import.meta.url), { workerData: { directory, test }, resourceLimits: { maxOldGenerationSizeMb: 4096 } });
    let finished = false;
    const done = value => { if (finished) return; finished = true; clearTimeout(timer); resolve(value); };
    const timer = setTimeout(() => { done({ pass: false, why: "timeout" }); worker.terminate(); }, 120000);
    worker.on("message", r => {
      const errors = (r.diagnostics ?? []).filter(d => d.severity === "error");
      const output = (r.stdout || "") + (r.stderr || "") + (r.rendered || "") + "\n" + errors.map(d => d.message).join("\n");
      const negative = test.mode === "check-fail" || "error_contains" in test;
      let pass;
      if (test.mode === "check-fail") {
        if (!test.diagnostic_contains?.trim()) { done({ pass: false, why: "missing precise diagnostic" }); worker.terminate(); return; }
        pass = r.exitCode != null && r.exitCode !== 0 && output.includes(test.diagnostic_contains);
      } else if ("error_contains" in test) {
        pass = r.exitCode === test.exit_code && output.includes(test.error_contains);
      } else {
        pass = r.exitCode === 0 && errors.length === 0 && (!("stdout" in test) || r.stdout === test.stdout);
      }
      if (!negative && !pass && errors.length && errors.every(d => ["unavailable", "unknown-library"].includes(d.code))) {
        done({ unavailable: errors.map(d => d.message).join("; ") });
      } else {
        if (process.env.PLAYGROUND_VERBOSE) console.log(test.id + "\n" + output);
        done({ pass, why: pass ? "" : "exit " + r.exitCode + "; " + (errors[0]?.message || r.stderr || "stdout differs") });
      }
      worker.terminate();
    });
    worker.on("error", e => done({ pass: false, why: "crash: " + e.message }));
    worker.on("exit", () => done({ pass: false, why: "worker exited without result" }));
  });
  const results = new Map();
  let next = 0;
  await Promise.all(Array.from({ length: 4 }, async () => { while (next < tests.length) { const c = tests[next++]; results.set(c.id, await runOne(c)); } }));
  for (const test of tests) {
    if (test.positive_baseline && !results.get("contracts/" + test.positive_baseline)?.pass) {
      results.set(test.id, { pass: false, why: "positive public baseline did not pass" });
    }
  }
  const failed = tests.filter(c => !results.get(c.id).pass && !results.get(c.id).unavailable);
  const unavailable = tests.filter(c => results.get(c.id).unavailable);
  const passed = tests.filter(c => results.get(c.id).pass);
  for (const c of failed) console.log("FAIL " + c.id + ": " + results.get(c.id).why);
  for (const c of unavailable) console.log("UNAVAILABLE " + c.id + ": " + results.get(c.id).unavailable);
  const missing = only.filter(id => !catalogue.some(c => c.id === id));
  if (missing.length) console.log("no such case: " + missing.join(", "));
  console.log(passed.length + " passed (including expected precise rejections), " + failed.length + " failed, " + unavailable.length + " native-only unavailable; " + tests.length + " selected");
  if (failed.length || missing.length || !tests.length) process.exit(1);
}
