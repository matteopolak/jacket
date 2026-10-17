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
test("required rejection matches a diagnosed failure before program exit", () => {
  const contract = { mode: "check-fail", diagnostic_contains: "missing http_date_now" };
  const diagnostics = [{ severity: "error", message: "missing http_date_now" }];
  assert.equal(playgroundContract(contract, { exitCode: null, failurePhase: "compile", diagnostics }).pass, true);
  for (const exitCode of [0, 1, undefined]) assert.equal(playgroundContract(contract, { exitCode, diagnostics }).pass, false);
  assert.equal(playgroundContract(contract, { exitCode: null, failurePhase: "compile", diagnostics: [{ severity: "error", message: "unrelated syntax error" }] }).pass, false);
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
test("termination stdout protocols retain nonzero status and empty stderr", () => {
  const contract = { exit_code: 37, stdout: "before exit 37\n", stderr: "" };
  assert.equal(playgroundContract(contract, { exitCode: 37, stdout: contract.stdout, stderr: "" }).pass, true);
  for (const exitCode of [0, 1, null, undefined]) assert.equal(playgroundContract(contract, { exitCode, stdout: contract.stdout }).pass, false);
  assert.equal(playgroundContract(contract, { exitCode: 37, stdout: contract.stdout, stderr: "unexpected" }).pass, false);
  assert.equal(playgroundContract(contract, { exitCode: 37, stdout: contract.stdout + "continued\n" }).pass, false);
});

test("browser compilation failures retain exact diagnostics despite a null program exit", () => {
  const failure = { exitCode: null, failurePhase: "compile", diagnostics: [{ severity: "error", message: "precise compiler rejection" }] };
  assert.equal(playgroundContract({ mode: "check-fail", diagnostic_contains: "precise compiler rejection" }, failure).pass, true);
  assert.equal(playgroundContract({ exit_code: 1, error_contains: "precise compiler rejection" }, failure).pass, true);
  assert.equal(playgroundContract({ exit_code: 37, error_contains: "precise compiler rejection" }, failure).pass, false);
  assert.equal(playgroundContract({ mode: "check-fail", diagnostic_contains: "different rejection" }, failure).pass, false);
  assert.equal(playgroundContract({ mode: "check-fail", diagnostic_contains: "precise compiler rejection" }, { exitCode: null, rendered: "precise compiler rejection" }).pass, false);
  assert.equal(playgroundContract({ mode: "check-fail", diagnostic_contains: "precise compiler rejection" }, { exitCode: null, rendered: "source contains precise compiler rejection", diagnostics: [{ severity: "error", message: "unrelated rejection" }] }).pass, false);
});

test("runtime and input errors cannot satisfy compile-rejection contracts", () => {
  const contract = { mode: "check-fail", diagnostic_contains: "precise rejection" };
  for (const failurePhase of ["runtime", "input", null, undefined]) {
    assert.equal(playgroundContract(contract, { failurePhase, exitCode: null, diagnostics: [{ severity: "error", message: "precise rejection" }] }).pass, false);
  }
});
