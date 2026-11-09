// Runs example programs in the browser compiler (`jacket doc --runtime`), the way the Run button's
// worker does: the same engine, one `main.jai`, the same step budget.
//
//   node run_examples.mjs <runtime dir> <programs.json> <shard> <shards>
//
// `programs.json` is an array of `{ text, timeoutSeconds }`; `timeoutSeconds` is a decimal string,
// so a budget larger than a double holds still reads exactly. Prints one JSON line per program of
// this shard: `{ i, ok, message }`.
//
// The compiler runs synchronously, so each shard compiles in a worker thread: the main thread can
// stop a program that outlives its timeout, and the shard goes on with a fresh worker.
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";

/** The longest delay `setTimeout` takes. */
const LONGEST_DELAY = 2_147_483_647;

/** The worker: one engine, kept for every program that does not crash it. */
async function compile() {
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

/** What `worker` answers for `program`, or the reason it was stopped; `retire` when the worker is no use after. */
function run(worker, program) {
  const seconds = program.timeoutSeconds ?? "60";
  if (!/^(0|[1-9]\d*)$/u.test(seconds)) throw new Error(`invalid example timeout ${seconds}`);
  return new Promise(done => {
    const finish = (line, retire) => {
      clearTimeout(timer);
      worker.off("message", onMessage);
      worker.off("exit", onExit);
      done({ line, retire });
    };
    const onMessage = line => finish(line, false);
    const onExit = code =>
      finish({ ok: false, message: `The compiler stopped without a result (status ${code}).` }, true);
    const delay = Math.min(Number(seconds) * 1000, LONGEST_DELAY);
    const timer = setTimeout(
      () => finish({ ok: false, message: `Stopped after ${seconds}s.` }, true),
      delay,
    );
    worker.once("message", onMessage);
    worker.once("exit", onExit);
    worker.postMessage(program.text);
  });
}

async function main() {
  const [runtime, file, shard, shards] = process.argv.slice(2);
  const programs = JSON.parse(await readFile(file, "utf8"));
  let worker = null;
  try {
    for (let i = Number(shard); i < programs.length; i += Number(shards)) {
      worker ??= new Worker(new URL(import.meta.url), { workerData: { runtime: resolve(runtime) } });
      const { line, retire } = await run(worker, programs[i]);
      if (retire) {
        await worker.terminate();
        worker = null;
      }
      process.stdout.write(JSON.stringify({ i, ...line }) + "\n");
    }
  } finally {
    if (worker) await worker.terminate();
  }
}

if (isMainThread) await main();
else await compile();
