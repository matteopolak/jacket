import test from "node:test";
import assert from "node:assert/strict";
import { playgroundContract } from "./playground_contract.mjs";

test("supported missing providers remain failed executions", () => {
  for (const code of ["unknown-library", "unavailable"]) {
    const result = playgroundContract({}, { exitCode: 1, diagnostics: [{ severity: "error", code, message: "missing actual provider" }] });
    assert.equal(result.pass, false);
    assert.equal(result.unavailable, undefined);
  }
});
test("required rejection matches exact diagnostic and nonzero execution", () => {
  const contract = { mode: "check-fail", diagnostic_contains: "missing http_date_now" };
  assert.equal(playgroundContract(contract, { exitCode: 1, stderr: "missing http_date_now" }).pass, true);
  for (const exitCode of [0, null, undefined]) assert.equal(playgroundContract(contract, { exitCode, stderr: "missing http_date_now" }).pass, false);
  assert.equal(playgroundContract(contract, { exitCode: 1, stderr: "unrelated syntax error" }).pass, false);
});
test("missing negative diagnostic never blesses an arbitrary failure", () => {
  assert.equal(playgroundContract({ mode: "check-fail" }, { exitCode: 1 }).pass, false);
});
test("positive stdout and error diagnostics stay strict", () => {
  const contract = { stdout: "7\n" };
  assert.equal(playgroundContract(contract, { exitCode: 0, stdout: "7\n" }).pass, true);
  assert.equal(playgroundContract(contract, { exitCode: 0, stdout: "8\n" }).pass, false);
  assert.equal(playgroundContract(contract, { exitCode: 0, stdout: "7\n", diagnostics: [{ severity: "error", message: "bad" }] }).pass, false);
});
test("explicit failure contracts retain exact exit status", () => {
  const contract = { exit_code: 37, error_contains: "expected failure" };
  assert.equal(playgroundContract(contract, { exitCode: 37, stderr: "expected failure" }).pass, true);
  assert.equal(playgroundContract(contract, { exitCode: 1, stderr: "expected failure" }).pass, false);
});
