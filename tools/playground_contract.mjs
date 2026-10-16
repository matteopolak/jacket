// Result accounting only. Capability selection belongs to the canonical catalogue.
export function playgroundContract(test, result) {
  const errors = (result.diagnostics ?? []).filter(d => d.severity === "error");
  const output = (result.stdout || "") + (result.stderr || "") + (result.rendered || "") + "\n" + errors.map(d => d.message).join("\n");
  if (test.mode === "check-fail") {
    if (!test.diagnostic_contains?.trim()) return { pass: false, output, why: "missing precise diagnostic" };
    const pass = result.exitCode != null && result.exitCode !== 0 && output.includes(test.diagnostic_contains);
    return { pass, output, why: pass ? "" : "required precise rejection did not occur" };
  }
  const pass = "error_contains" in test
    ? result.exitCode === test.exit_code && output.includes(test.error_contains)
    : result.exitCode === (test.exit_code ?? 0) && errors.length === 0 && (!("stdout" in test) || result.stdout === test.stdout) && (!("stderr" in test) || (result.stderr || "") === test.stderr);
  return { pass, output, why: pass ? "" : "exit " + result.exitCode + "; " + (errors[0]?.message || result.stderr || "stdout differs") };
}
