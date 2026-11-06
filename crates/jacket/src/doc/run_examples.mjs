// Runs example programs in the browser compiler (`jacket doc --runtime`), the way the Run button's
// worker does: the same engine, one `main.jai`, the same step budget.
//
//   node run_examples.mjs <runtime dir> <programs.json> <shard> <shards>
//
// `programs.json` is an array of `{ text, timeoutSeconds }`; decimal strings retain u64 budgets.
// Prints one JSON line per program of this shard:
// `{ i, ok, message }`.
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";

/** Retain one compiler between successful programs; a terminated worker owns no shared engine. */
async function workerMain() {
  const { runtime } = workerData;
  const { createEngine } = await import(pathToFileURL(resolve(runtime, "engine.mjs")).href);
  const bytes = await readFile(join(runtime, "jai_wasm.wasm"));
  let engine = null;
  parentPort.on("message", async text => {
    let line;
    try {
      engine ??= await createEngine(bytes);
      const r = engine.play({ "main.jai": text }, "main.jai", { budget: 200_000_000 });
      const printed = (r.stdout || "") + (r.stderr || "");
      const errors = (r.diagnostics || []).some(d => d.severity === "error");
      if (r.exitCode === 0 && !errors) {
        line = { ok: true, message: "" };
      } else {
        const why = r.rendered || printed ||
          (r.exitCode !== null ? `The example exited with status ${r.exitCode}.` : "The example did not compile.");
        line = { ok: false, message: why.trim() };
      }
    } catch (error) {
      if (error && error.compilerCrashed) engine = null;
      line = { ok: false, message: String((error && error.message) || error) };
    }
    parentPort.postMessage(line);
  });
}

/** Supervise synchronous execution from a separate event loop; initialization shares the deadline. */
function runProgram(worker, program) {
  const seconds = program.timeoutSeconds ?? "60";
  if (typeof seconds !== "string" || !/^(0|[1-9]\d*)$/u.test(seconds)) throw new Error("Invalid example timeout");
  const deadline = process.hrtime.bigint() + BigInt(seconds) * 1_000_000_000n;
  return new Promise(resolve => {
    let settled = false, timer;
    function finish(line, retire = false) {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      worker.off("message", onMessage);
      worker.off("exit", onExit);
      // Keep the error listener until retirement completes, so teardown races stay owned.
      resolve({ line, retire, cleanup: () => worker.off("error", onError) });
    }
    function expired() { finish({ ok: false, message: `Stopped after ${seconds}s.` }, true); }
    function schedule() {
      const remaining = deadline - process.hrtime.bigint();
      if (remaining <= 0n) { expired(); return; }
      const milliseconds = (remaining + 999_999n) / 1_000_000n;
      timer = setTimeout(schedule, Number(milliseconds > 2_147_483_647n ? 2_147_483_647n : milliseconds));
    }
    function onMessage(line) {
      if (process.hrtime.bigint() >= deadline) { expired(); return; }
      if (!line || typeof line.ok !== "boolean" || typeof line.message !== "string") {
        finish({ ok: false, message: "Invalid compiler worker response." }, true);
      } else finish(line);
    }
    function onError(error) { finish({ ok: false, message: String(error.message || error) }, true); }
    function onExit(code) { finish({ ok: false, message: `The compiler worker exited without a result (status ${code}).` }, true); }
    worker.once("message", onMessage);
    worker.once("error", onError);
    worker.once("exit", onExit);
    schedule();
    if (!settled) worker.postMessage(program.text);
  });
}

/** One shard keeps its engine until timeout/crash; every owned thread is reaped before exit. */
async function main() {
  const [runtime, file, shard, shards] = process.argv.slice(2);
  const programs = JSON.parse(await readFile(file, "utf8"));
  let worker;
  try {
    for (let i = Number(shard); i < programs.length; i += Number(shards)) {
      worker ??= new Worker(new URL(import.meta.url), { workerData: { runtime: resolve(runtime) } });
      const result = await runProgram(worker, programs[i]);
      try {
        if (result.retire) { await worker.terminate(); worker = undefined; }
      } finally { result.cleanup(); }
      process.stdout.write(JSON.stringify({ i, ...result.line }) + "\n");
    }
  } finally {
    if (worker) await worker.terminate();
  }
}

if (isMainThread) await main();
else await workerMain();
