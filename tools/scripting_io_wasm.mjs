// Real sandbox I/O behavior, independent of public native-only module facades.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

export async function checkScriptingIo(engine, createLiveEngine) {
  // Arguments and standard input: fixed (`stdin` option) and, with JSPI, supplied by the page as the
  // program reads (`jai_stdin_read` waits for a line, answers 0 at the end of input).
  const echo = {
    "main.jai": await readFile(new URL("../tests/compiler/language/browser-io/stdin.jai", import.meta.url), "utf8"),
  };
  const fixed = engine.play(echo, "main.jai", { args: ["main", "a b"], stdin: "hello\n" });
  assert.deepEqual(fixed.diagnostics, [], fixed.rendered);
  assert.equal(fixed.exitCode, 0, fixed.rendered);
  assert.equal(fixed.stdout, "argc=2 last=a b\ngot hello\nthen true\n", fixed.rendered);
  if (engine.jspi) {
    const lines = ["typed\n", ""];
    const waited = [];
    const live = await createLiveEngine({ host: { functions: {
      jai_stdin_read([pointer, capacity], memory) {
        const line = new TextEncoder().encode(lines.shift() ?? "");
        waited.push(line.length);
        return new Promise(resolve => setTimeout(() => {
          new Uint8Array(memory.buffer, Number(pointer), Number(capacity)).set(line);
          resolve(line.length);
        }, 5));
      },
    } } });
    const interactive = await live.playAsync(echo, "main.jai", { args: ["main"] });
    assert.deepEqual(interactive.diagnostics, [], interactive.rendered);
    assert.equal(interactive.exitCode, 0, interactive.rendered);
    assert.equal(interactive.stdout, "argc=1 last=main\ngot typed\nthen true\n", interactive.rendered);
    assert.deepEqual(waited, [6, 0]);
    console.log("PASS: asynchronous JSPI input");
  } else {
    console.log("NOT RUN: asynchronous input requires JSPI");
  }
  console.log("PASS: arguments and standard input");

  // The supported C exit bridge retains output and returns its supplied status.
  const exited = engine.play({ "main.jai": await readFile(new URL("../tests/compiler/language/browser-io/exit.jai", import.meta.url), "utf8") }, "main.jai");
  assert.equal(exited.exitCode, 7, exited.rendered);
  assert.equal(exited.stdout, "before\n");
  assert.deepEqual(exited.diagnostics, []);
  console.log("PASS: exit");
}
