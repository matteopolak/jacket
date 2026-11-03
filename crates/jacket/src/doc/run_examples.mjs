// Runs example programs in the browser compiler (`jacket doc --runtime`), the way the Run button's
// worker does: the same engine, one `main.jai`, the same step budget.
//
//   node run_examples.mjs <runtime dir> <programs.json> <shard> <shards>
//
// `programs.json` is an array of `{ text }`. Prints one JSON line per program of this shard:
// `{ i, ok, message }`.
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [runtime, file, shard, shards] = process.argv.slice(2);
const { createEngine } = await import(pathToFileURL(resolve(runtime, "engine.mjs")).href);
const bytes = await readFile(join(runtime, "jai_wasm.wasm"));
const programs = JSON.parse(await readFile(file, "utf8"));
let engine = null;

for (let i = Number(shard); i < programs.length; i += Number(shards)) {
  let line;
  try {
    engine ??= await createEngine(bytes);
    const r = engine.play({ "main.jai": programs[i].text }, "main.jai", { budget: 200_000_000 });
    const printed = (r.stdout || "") + (r.stderr || "");
    const errors = (r.diagnostics || []).some(d => d.severity === "error");
    if (r.exitCode === 0 && !errors) {
      line = { i, ok: true, message: "" };
    } else {
      const why = r.rendered || printed ||
        (r.exitCode !== null ? `The example exited with status ${r.exitCode}.` : "The example did not compile.");
      line = { i, ok: false, message: why.trim() };
    }
  } catch (error) {
    if (error && error.compilerCrashed) engine = null;
    line = { i, ok: false, message: String((error && error.message) || error) };
  }
  process.stdout.write(JSON.stringify(line) + "\n");
}
