// Runs the example programs listed in tests/examples.json (examples/tour, ...) in a wasm engine, the
// way the hosted playground does: every file of the example's directory as the workspace, its main
// file as the entry point, and the case's `budget` (the playground's basic-block limit).
import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

export async function exampleCases() {
  return JSON.parse(await readFile(path.join(root, "tests/examples.json"), "utf8")).cases;
}

/** Every regular file under `directory`, keyed by its relative POSIX path. */
export async function workspaceFiles(directory, base = directory, files = {}) {
  for (const entry of (await readdir(directory, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) await workspaceFiles(full, base, files);
    else if (entry.isFile()) files[path.relative(base, full).split(path.sep).join("/")] = await readFile(full, "utf8");
  }
  return files;
}

/** Runs one case and checks its output; returns the wall-clock milliseconds the run took. */
export function checkExample(engine, testCase, files) {
  const started = performance.now();
  const result = engine.play(files, testCase.main, { budget: testCase.budget });
  const milliseconds = performance.now() - started;
  const errors = result.diagnostics.filter(item => item.severity === "error");
  assert.deepEqual(errors, [], `${testCase.id}: ${JSON.stringify(errors)}`);
  assert.equal(result.exitCode, 0, `${testCase.id}: exit code\n${result.stderr}`);
  for (const line of testCase.stdout_contains) assert(result.stdout.includes(line), `${testCase.id}: stdout lacks ${JSON.stringify(line)}`);
  for (const text of testCase.stdout_excludes ?? []) assert(!result.stdout.includes(text), `${testCase.id}: stdout has ${JSON.stringify(text)}`);
  return milliseconds;
}

/** Source-tree example cases, for checks that run a freshly built module. */
export async function checkSourceExamples(engine) {
  const timings = [];
  for (const testCase of await exampleCases()) {
    const files = await workspaceFiles(path.join(root, testCase.directory));
    timings.push(`${testCase.id} ${Math.round(checkExample(engine, testCase, files))} ms`);
  }
  return timings;
}
