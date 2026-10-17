import assert from "node:assert/strict";
import { test } from "node:test";
import { playgroundCaseResult } from "./playground_case_result.mjs";

test("compiler diagnostics never promote a required browser case to unavailable", () => {
  for (const code of ["unknown-library", "unavailable", "unknown-identifier"]) {
    const result = playgroundCaseResult({}, { exitCode: 1, diagnostics: [{ severity: "error", code, message: "missing provider" }] });
    assert.equal(result.pass, false);
    assert.equal(result.unavailable, undefined);
  }
});

test("mode-specific negatives require their precise failure and reject crashes", () => {
  const spec = { mode: "check-fail", diagnostic_contains: "missing public member" };
  assert.equal(playgroundCaseResult(spec, { exitCode: null, failurePhase: "compile", diagnostics: [{ severity: "error", message: "missing public member" }] }).pass, true);
  for (const value of [
    { exitCode: 0, rendered: "missing public member" },
    { exitCode: null, rendered: "missing public member" },
    { exitCode: 1, rendered: "unrelated diagnostic" },
    { compilerCrashed: true, requestFailure: "missing public member" },
    { compilerCrashed: true, exitCode: 1, rendered: "missing public member" },
    { requestFailure: "input byte limit exceeded" },
  ]) assert.equal(playgroundCaseResult(spec, value).pass, false);
  assert.equal(playgroundCaseResult({ mode: "check-fail" }, { exitCode: 1 }).pass, false);
});

test("positive stdout and explicit runtime rejection oracles stay exact", () => {
  assert.equal(playgroundCaseResult({ stdout: "ok\n" }, { exitCode: 0, stdout: "ok\n" }).pass, true);
  assert.equal(playgroundCaseResult({ stdout: "ok\n" }, { exitCode: 0, stdout: "wrong" }).pass, false);
  const expected = { exit_code: 7, error_contains: "specific failure" };
  assert.equal(playgroundCaseResult(expected, { exitCode: 7, stderr: "specific failure" }).pass, true);
  assert.equal(playgroundCaseResult(expected, { exitCode: 1, stderr: "specific failure" }).pass, false);
});


test("actual worker host keeps diagnostic failures and predeclared availability separate", async () => {
  const { mkdtemp, mkdir, writeFile, copyFile, rm } = await import("node:fs/promises");
  const { tmpdir } = await import("node:os");
  const path = (await import("node:path")).default;
  const { fileURLToPath } = await import("node:url");
  const { spawnSync } = await import("node:child_process");
  const base = await mkdtemp(path.join(tmpdir(), "jai-browser-policy-"));
  try {
    const tools = path.join(base, "tools"), stage = path.join(base, "stage");
    await mkdir(tools); await mkdir(stage); await mkdir(path.join(base, "tests/inert"), { recursive: true });
    const cases = [
      { id: "required", result: { exitCode: 1, diagnostics: [{ severity: "error", code: "unknown-library", message: "required provider missing" }] } },
      { id: "unavailable-diagnostic", result: { exitCode: 1, diagnostics: [{ severity: "error", code: "unavailable", message: "required capability missing" }] } },
      { id: "negative", mode: "check-fail", diagnostic_contains: "exact rejection", result: { exitCode: null, failurePhase: "compile", diagnostics: [{ severity: "error", message: "exact rejection" }] } },
      { id: "wrong-negative", mode: "check-fail", diagnostic_contains: "exact rejection", result: { exitCode: 1, rendered: "unrelated rejection" } },
      { id: "crash", mode: "check-fail", diagnostic_contains: "exact rejection", result: { requestFailure: "exact rejection" } },
      { id: "boundary", result: { requestFailure: "input byte limit exceeded" } },
      { id: "native", browser_unavailable_reason: "reviewed native capability", result: { requestFailure: "must not execute" } },
    ].map(c => ({ family: "compiler", browser_unavailable_reason: null, ...c, path: `tests/inert/${c.id}.jai` }));
    // Authored transport stub: no compiler, SDK, Jai program or archive release is executed.
    const catalogue = JSON.stringify(cases);
    const script = "import json\nprint(json.dumps(json.loads(" + JSON.stringify(catalogue) + ")))\n";
    await writeFile(path.join(tools, "playground_policy.py"), script);
    await writeFile(path.join(tools, "stdlib_test_catalog.py"), script);
    for (const name of ["check_playground_stdlib.mjs", "playground_request.mjs", "playground_case_result.mjs", "playground_contract.mjs"]) {
      await copyFile(fileURLToPath(new URL(name, import.meta.url)), path.join(tools, name));
    }
    for (const c of cases) await writeFile(path.join(base, c.path), JSON.stringify(c.result));
    await writeFile(path.join(stage, "jai_wasm.wasm"), "inert artifact, never instantiated");
    await writeFile(path.join(stage, "engine.mjs"), `export async function createEngine() { return { play(files, main) { const r = JSON.parse(files[main]); if (r.requestFailure) throw new Error(r.requestFailure); return r; } }; }`);
    const run = spawnSync(process.execPath, [path.join(tools, "check_playground_stdlib.mjs"), stage], { encoding: "utf8", timeout: 30000 });
    assert.equal(run.error, undefined);
    assert.equal(run.status, 1, run.stderr);
    assert.match(run.stdout, /1 passed .* 5 failed, 1 target\/mode unavailable; 7 selected/);
    for (const id of ["required", "unavailable-diagnostic", "wrong-negative", "crash", "boundary"]) assert.match(run.stdout, new RegExp("FAIL " + id + ":"));
    assert.match(run.stdout, /UNAVAILABLE native: reviewed native capability/);
    assert.match(run.stdout, /input byte limit exceeded; input files=/);
  } finally { await rm(base, { recursive: true, force: true }); }
});

test("rejected supplied-library requests report every counted channel", () => {
  const result = playgroundCaseResult({}, { requestFailure: "input byte limit exceeded", request: { fileCount: 1, pathBytes: 2, sourceBytes: 3, mainBytes: 4, libraryRootBytes: 6, totalBytes: 15 } });
  assert.equal(result.pass, false);
  assert.match(result.why, /bytes=15 .*libraryRoot=6/);
});
