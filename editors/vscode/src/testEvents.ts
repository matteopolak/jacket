// What `jacket test --format json` prints, one object per line (docs/tools/jacket-test.md,
// "Output formats"), and what the editor makes of it. Pure: nothing here knows about VS Code.

/** A test as `--list` prints it; `kind` is `test`, `golden`, `compile_fail`, `doc`, `bench` or `subtest`. */
export interface Listed {
  name: string;
  kind: string;
  file: string;
  /** 1-based. */
  line: number;
  notes: string[];
}

/** A failed check: where it was written and what it said. */
export interface Failure {
  test: string;
  file: string;
  line: number;
  message: string;
}

export interface SubtestResult {
  name: string;
  status: string;
  /** Seconds. */
  duration: number;
}

export type Finish = "passed" | "failed" | "skipped" | "timed_out" | "xfail" | "xpass" | "flaky";

/** The event that ends a test. */
export interface Finished {
  name: string;
  kind: string;
  file: string;
  line: number;
  finish: Finish;
  /** Seconds. */
  duration: number;
  failures: Failure[];
  problems: string[];
  subtests: SubtestResult[];
  stdout: string;
  stderr: string;
  reason: string | undefined;
}

export type Event =
  | { type: "list"; test: Listed }
  | { type: "started"; name: string }
  | { type: "finished"; test: Finished }
  | { type: "run-started"; tests: number }
  | { type: "run-finished"; failed: number };

const FINISHES: readonly string[] = ["passed", "failed", "skipped", "timed_out", "xfail", "xpass", "flaky"];

function text(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function number(value: unknown): number {
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function records(value: unknown): Record<string, unknown>[] {
  return Array.isArray(value) ? value.filter((v): v is Record<string, unknown> => typeof v === "object" && v !== null) : [];
}

/** The event a line of standard output holds; undefined for anything else (warnings, the test program's own text). */
export function parseEvent(line: string): Event | undefined {
  if (!line.startsWith("{")) return undefined;
  let value: Record<string, unknown>;
  try {
    value = JSON.parse(line) as Record<string, unknown>;
  } catch {
    return undefined;
  }
  const name = text(value.name);
  if (value.type === "list") {
    const notes = Array.isArray(value.notes) ? value.notes.map(text) : [];
    return { type: "list", test: { name, kind: text(value.kind), file: text(value.file), line: number(value.line), notes } };
  }
  if (value.type === "run") {
    if (value.event === "started") return { type: "run-started", tests: number(value.tests) };
    if (value.event === "finished") return { type: "run-finished", failed: number(value.failed) };
    return undefined;
  }
  if (value.type !== "test") return undefined;
  if (value.event === "started") return { type: "started", name };
  const event = text(value.event);
  if (!FINISHES.includes(event)) return undefined;
  return {
    type: "finished",
    test: {
      name,
      kind: text(value.kind),
      file: text(value.file),
      line: number(value.line),
      finish: event as Finish,
      duration: number(value.duration),
      failures: records(value.failures).map((f) => ({ test: text(f.test), file: text(f.file), line: number(f.line), message: text(f.message) })),
      problems: Array.isArray(value.problems) ? value.problems.map(text) : [],
      subtests: records(value.subtests).map((s) => ({ name: text(s.name), status: text(s.status), duration: number(s.duration) })),
      stdout: text(value.stdout),
      stderr: text(value.stderr),
      reason: typeof value.reason === "string" ? value.reason : undefined,
    },
  };
}

export type Verdict = "passed" | "failed" | "skipped";

/** One thing to show about a failure; `file` and `line` (1-based) are where jacket says it happened. */
export interface Message {
  text: string;
  /** The test or subtest the failure belongs to, when jacket says. */
  test?: string;
  file?: string;
  line?: number;
}

export interface Outcome {
  verdict: Verdict;
  durationMs: number;
  messages: Message[];
}

/** `passed`, `xfail` (failed as expected) and `flaky` (passed on a retry) are passes; the rest that is not a skip fails. */
export function verdictOf(finish: Finish): Verdict {
  switch (finish) {
    case "passed":
    case "xfail":
    case "flaky":
      return "passed";
    case "skipped":
      return "skipped";
    default:
      return "failed";
  }
}

export function subtestVerdict(status: string): Verdict {
  return status === "pass" ? "passed" : status === "skip" ? "skipped" : "failed";
}

/** The verdict, duration and messages of a finished test. */
export function outcomeOf(test: Finished): Outcome {
  const verdict = verdictOf(test.finish);
  const messages: Message[] = [];
  if (verdict === "failed") {
    for (const failure of test.failures) messages.push({ text: failure.message, test: failure.test, file: failure.file, line: failure.line });
    for (const problem of test.problems) messages.push({ text: problem });
    if (test.finish === "xpass") messages.push({ text: "the test passed but is marked @xfail; remove the note" });
    if (test.finish === "timed_out" && messages.length === 0) messages.push({ text: "the test timed out" });
    // A crash has no failed check: its last words are the best explanation there is.
    if (messages.length === 0 || (test.failures.length === 0 && test.stderr.trim())) {
      const stderr = test.stderr.trim();
      messages.push({ text: stderr || "the test failed" });
    }
  }
  return { verdict, durationMs: test.duration * 1000, messages };
}

/** The failures of a run that have a place in a source file, as the editor's problems list shows them. */
export function located(messages: readonly Message[]): (Message & { file: string; line: number })[] {
  return messages.filter((m): m is Message & { file: string; line: number } => m.file !== undefined && m.line !== undefined && m.line > 0);
}

/** The test's output as the run's terminal shows it: the text with `\r\n` line ends. */
export function terminalText(value: string): string {
  return value.replace(/\r?\n/g, "\r\n");
}
