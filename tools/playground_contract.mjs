// Result accounting only. Capability selection belongs to the canonical catalogue.
export function playgroundContract(test, result) {
  const errors = (result.diagnostics ?? []).filter(d => d.severity === "error");
  const output = (result.stdout || "") + (result.stderr || "") + (result.rendered || "") + "\n" + errors.map(d => d.message).join("\n");
  // Null is not a process exit. A diagnosed phase identifies compiler/runtime failure.
  const diagnosedFailure = result.exitCode === null && errors.length > 0 && ["compile", "runtime"].includes(result.failurePhase);
  const hasDiagnostic = text => errors.some(d => d.message.includes(text));
  if (test.mode === "check-fail") {
    if (!test.diagnostic_contains?.trim()) return { pass: false, output, why: "missing precise diagnostic" };
    const pass = diagnosedFailure && result.failurePhase === "compile" && hasDiagnostic(test.diagnostic_contains);
    return { pass, output, why: pass ? "" : "required precise rejection did not occur" };
  }
  const pass = "error_contains" in test
    ? (diagnosedFailure
      ? test.exit_code === 1 && hasDiagnostic(test.error_contains)
      : result.exitCode === test.exit_code && output.includes(test.error_contains))
    : result.exitCode === (test.exit_code ?? 0) && errors.length === 0 && (!("stdout" in test) || result.stdout === test.stdout) && (!("stderr" in test) || (result.stderr || "") === test.stderr);
  return { pass, output, why: pass ? "" : "exit " + result.exitCode + "; " + (errors[0]?.message || result.stderr || "stdout differs") };
}
