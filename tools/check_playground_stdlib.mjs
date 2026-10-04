// Runs every tests/stdlib/*.jai through the browser engine (a fresh wasm instance per test) and compares the
// pass set with tools/playground_stdlib_expected.json. Usage:
//   node tools/check_playground_stdlib.mjs <staged-dir> [--update] [--report] [name.jai...]
// A test passes when it finishes with exit code 0 and no error diagnostics, the same bar as the native loop
// (`jaic run` exits 0). Tests that cannot run in a browser are listed with a reason under "excluded".
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const testDir = path.join(root, "tests/stdlib");
const expectedPath = path.join(here, "playground_stdlib_expected.json");

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
  const flags = new Set(argv.filter(a => a.startsWith("--")));
  const positional = argv.filter(a => !a.startsWith("--"));
  if (!positional[0]) throw new Error("usage: node tools/check_playground_stdlib.mjs <staged-dir> [--update] [--report] [test.jai...]");
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
  if (flags.has("--report") || flags.has("--update") || only.length) for (const n of failed) console.log(`FAIL ${n}: ${results.get(n).why}`);
  console.log(`${passed.length} of ${names.length} stdlib tests pass in the playground`);
  if (flags.has("--update")) {
    // Reasons are written by hand: keep the existing ones, flag new failures for a human to explain.
    const previous = JSON.parse(await readFile(expectedPath, "utf8")).excluded ?? {};
    const excluded = Object.fromEntries(failed.map(n => [n, previous[n] ?? `TODO explain: ${results.get(n).why}`]));
    await writeFile(expectedPath, JSON.stringify({ pass: passed, excluded }, null, 1) + "\n");
  } else if (!only.length) {
    const expected = JSON.parse(await readFile(expectedPath, "utf8"));
    const want = new Set(expected.pass), got = new Set(passed), excluded = new Set(Object.keys(expected.excluded));
    const regressed = [...want].filter(n => !got.has(n)), gained = [...got].filter(n => !want.has(n));
    const unlisted = names.filter(n => !want.has(n) && !excluded.has(n));
    const stale = [...excluded, ...want].filter(n => !names.includes(n));
    if (regressed.length || gained.length || unlisted.length || stale.length) {
      console.error(`playground stdlib pass set changed. regressed: [${regressed}] newly passing: [${gained}] unlisted: [${unlisted}] missing from tests/stdlib: [${stale}]. Explain each exclusion in tools/playground_stdlib_expected.json (--update rewrites the pass list).`);
      process.exit(1);
    }
  }
}
