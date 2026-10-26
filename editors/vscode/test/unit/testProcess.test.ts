import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { Event } from "../../src/testEvents";
import { listArguments, runArguments, runJacket, withToolFolder } from "../../src/testProcess";

describe("jacket test arguments", () => {
  it("lists as JSON", () => {
    assert.deepEqual(listArguments(["--interp"]), ["test", "--list", "--format", "json", "--interp"]);
  });

  it("runs everything with no names", () => {
    assert.deepEqual(runArguments({}, []), ["test", "--format", "json"]);
  });

  it("runs the named tests whole, before the extra arguments", () => {
    assert.deepEqual(runArguments({ names: ["adds", "doc: M.f#1"] }, ["--", "-O2"]), ["test", "--format", "json", "--exact", "adds", "doc: M.f#1", "--", "-O2"]);
  });

  it("puts a lens' flags first", () => {
    assert.deepEqual(runArguments({ flags: ["--path", "/p", "-E", "test(=adds)"] }, []), ["test", "--format", "json", "--path", "/p", "-E", "test(=adds)"]);
  });
});

describe("the tool's folder on PATH", () => {
  it("goes first", () => {
    assert.equal(withToolFolder({ PATH: "/usr/bin" }, "/opt/jai/jaic", "linux").PATH, "/opt/jai:/usr/bin");
    assert.deepEqual(withToolFolder({ A: "1" }, undefined, "linux"), { A: "1" });
  });

  it("uses the Path variable and ; on Windows", () => {
    assert.equal(withToolFolder({ Path: "C:\\bin" }, "C:\\jai\\jaic.exe", "win32").Path, "C:\\jai;C:\\bin");
  });
});

describe("running jacket", () => {
  const script = [
    'process.stdout.write(\'{"type":"run","event":"started","tests":1}\\n{"type":"te\');',
    "setTimeout(() => {",
    '  process.stdout.write(\'st","event":"started","name":"a"}\\nnot an event\\n\');',
    "  process.stderr.write('jacket: root /p\\n');",
    "  process.exit(3);",
    "}, 20);",
  ].join("\n");

  it("delivers events whole as their lines complete, and reports the exit and standard error", async () => {
    const events: Event[] = [];
    const finish = await runJacket(process.execPath, ["-e", script], { cwd: process.cwd(), env: {}, onEvent: (e) => events.push(e) });
    assert.deepEqual(events, [
      { type: "run-started", tests: 1 },
      { type: "started", name: "a" },
    ]);
    assert.deepEqual(finish, { code: 3, stderr: "jacket: root /p\n" });
  });

  it("ends when aborted", async () => {
    const abort = new AbortController();
    const running = runJacket(process.execPath, ["-e", "setTimeout(() => {}, 60000)"], { cwd: process.cwd(), env: {}, onEvent: () => {}, signal: abort.signal });
    abort.abort();
    assert.equal((await running).code, null);
  });

  it("rejects when the program cannot start", async () => {
    await assert.rejects(runJacket("/no/such/jacket", [], { cwd: process.cwd(), env: {}, onEvent: () => {} }));
  });
});
