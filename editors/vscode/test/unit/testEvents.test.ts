import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { type Event, type Finished, located, outcomeOf, parseEvent, subtestVerdict, terminalText, verdictOf } from "../../src/testEvents";

function finished(line: object): Finished {
  const event: Event | undefined = parseEvent(JSON.stringify({ type: "test", ...line }));
  assert.equal(event?.type, "finished");
  return (event as Extract<Event, { type: "finished" }>).test;
}

const FAILED = {
  attempts: 1,
  duration: 0.0845,
  event: "failed",
  failures: [
    { file: "/p/tests/a.jai", line: 8, message: "expected true, got false", test: "fails" },
    { file: "/p/tests/a.jai", line: 9, message: "got:  3\nwant: 4", test: "fails" },
  ],
  file: "/p/tests/a.jai",
  kind: "test",
  line: 7,
  name: "fails",
  problems: [],
  reason: null,
  stderr: "",
  stdout: "",
  subtests: [],
  temp_dir: "/tmp/x",
};

describe("parsing events", () => {
  it("reads a list event", () => {
    const line = '{"file":"/p/tests/a.jai","kind":"subtest","line":24,"name":"single_values/1","notes":["cases(s64.[1,3,5])"],"type":"list"}';
    assert.deepEqual(parseEvent(line), {
      type: "list",
      test: { name: "single_values/1", kind: "subtest", file: "/p/tests/a.jai", line: 24, notes: ["cases(s64.[1,3,5])"] },
    });
  });

  it("reads the start of a run and of a test, and the end of a run", () => {
    assert.deepEqual(parseEvent('{"event":"started","tests":4,"type":"run"}'), { type: "run-started", tests: 4 });
    assert.deepEqual(parseEvent('{"event":"started","kind":"test","name":"passes","type":"test"}'), { type: "started", name: "passes" });
    assert.deepEqual(parseEvent('{"duration":0.5,"event":"finished","failed":2,"type":"run"}'), { type: "run-finished", failed: 2 });
  });

  it("reads a failed test with its failures", () => {
    const test = finished(FAILED);
    assert.equal(test.finish, "failed");
    assert.equal(test.name, "fails");
    assert.deepEqual(test.failures[1], { test: "fails", file: "/p/tests/a.jai", line: 9, message: "got:  3\nwant: 4" });
    assert.equal(test.reason, undefined);
  });

  it("ignores lines that are not events", () => {
    for (const line of ["", "warning: something", "{not json", '{"type":"test","event":"slow","name":"x","seconds":31}', '{"type":"bench"}']) {
      assert.equal(parseEvent(line), undefined, line);
    }
  });

  it("tolerates missing and mistyped fields", () => {
    const test = finished({ event: "passed", name: 3, duration: "fast", failures: [null, { line: "x" }], subtests: "none" });
    assert.equal(test.name, "");
    assert.equal(test.duration, 0);
    assert.equal(test.failures.length, 1);
    assert.deepEqual(test.subtests, []);
  });
});

describe("verdicts", () => {
  it("counts expected failures and retried passes as passes", () => {
    for (const event of ["passed", "xfail", "flaky"] as const) assert.equal(verdictOf(event), "passed");
    assert.equal(verdictOf("skipped"), "skipped");
    for (const event of ["failed", "timed_out", "xpass"] as const) assert.equal(verdictOf(event), "failed");
  });

  it("reads subtest statuses", () => {
    assert.deepEqual(["pass", "fail", "skip", "other"].map(subtestVerdict), ["passed", "failed", "skipped", "failed"]);
  });
});

describe("outcomes", () => {
  it("turns each failed check into a message with its place", () => {
    const outcome = outcomeOf(finished(FAILED));
    assert.equal(outcome.verdict, "failed");
    assert.ok(Math.abs(outcome.durationMs - 84.5) < 1e-9);
    assert.deepEqual(
      outcome.messages.map((m) => [m.text, m.file, m.line, m.test]),
      [
        ["expected true, got false", "/p/tests/a.jai", 8, "fails"],
        ["got:  3\nwant: 4", "/p/tests/a.jai", 9, "fails"],
      ],
    );
    assert.equal(located(outcome.messages).length, 2);
  });

  it("explains a crash with the program's last words", () => {
    const outcome = outcomeOf(
      finished({ ...FAILED, failures: [], problems: ["the test program exited with status 1 during the test"], stderr: "a.jai:17: error: array bounds check failed\n" }),
    );
    assert.deepEqual(
      outcome.messages.map((m) => m.text),
      ["the test program exited with status 1 during the test", "a.jai:17: error: array bounds check failed"],
    );
    assert.deepEqual(located(outcome.messages), []);
  });

  it("explains a timeout, an unexpected pass and a failure that says nothing", () => {
    assert.equal(outcomeOf(finished({ ...FAILED, event: "timed_out", failures: [] })).messages[0].text, "the test timed out");
    assert.match(outcomeOf(finished({ ...FAILED, event: "xpass", failures: [] })).messages[0].text, /xfail/);
    assert.equal(outcomeOf(finished({ ...FAILED, failures: [], problems: [] })).messages[0].text, "the test failed");
  });

  it("has no messages for a pass or a skip", () => {
    assert.deepEqual(outcomeOf(finished({ ...FAILED, event: "passed", failures: [], duration: 2 })), { verdict: "passed", durationMs: 2000, messages: [] });
    assert.equal(outcomeOf(finished({ ...FAILED, event: "skipped", failures: [], reason: "skipped on macos" })).verdict, "skipped");
  });
});

describe("terminal text", () => {
  it("ends lines with CRLF", () => {
    assert.equal(terminalText("a\nb\r\nc"), "a\r\nb\r\nc");
  });
});
