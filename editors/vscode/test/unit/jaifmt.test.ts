import assert from "node:assert/strict";
import * as path from "node:path";
import { describe, it } from "node:test";
import { isIgnored, minimalReplacement, runJaifmt } from "../../src/jaifmt";

function apply(text: string, change: ReturnType<typeof minimalReplacement>): string {
  return change ? text.slice(0, change.start) + change.text + text.slice(change.end) : text;
}

describe("minimal replacement", () => {
  it("keeps the common prefix and suffix", () => {
    const before = "main :: () {\nx:=1;\n}\n";
    const after = "main :: () {\n    x := 1;\n}\n";
    const change = minimalReplacement(before, after);
    assert.deepEqual(change, { start: 13, end: 16, text: "    x := " });
    assert.equal(apply(before, change), after);
  });

  it("is undefined when nothing changes", () => {
    assert.equal(minimalReplacement("a", "a"), undefined);
  });

  it("handles growth, shrinking and repeated text", () => {
    for (const [before, after] of [
      ["aaa", "aaaa"],
      ["aaaa", "a"],
      ["", "x\n"],
      ["x;  y;", "x;\ny;"],
      ["s := \"😀\";", "s := \"😁\";"],
    ]) {
      assert.equal(apply(before, minimalReplacement(before, after)), after, JSON.stringify([before, after]));
    }
  });

  it("does not split surrogate pairs", () => {
    const change = minimalReplacement("😀", "😁");
    assert.equal(change?.start, 0);
    assert.equal(change?.text, "😁");
  });
});

// With JAIFMT=<path to a built jaifmt>, also run the real formatter.
describe("jaifmt --stdin", { skip: !process.env.JAIFMT && "set JAIFMT to a jaifmt executable" }, () => {
  it("formats text and honours the jaifmt.toml of the working directory", async () => {
    const { mkdtempSync, writeFileSync } = await import("node:fs");
    const { tmpdir } = await import("node:os");
    const dir = mkdtempSync(path.join(tmpdir(), "jaifmt-"));
    const plain = await runJaifmt(process.env.JAIFMT!, "main::(){x:=1;}", dir);
    assert.equal(plain.ok, true, plain.stderr);
    assert.equal(plain.stdout, "main :: () {\n    x := 1;\n}\n");
    writeFileSync(path.join(dir, "jaifmt.toml"), "indent_width = 2\n");
    const two = await runJaifmt(process.env.JAIFMT!, "main::(){x:=1;}", dir);
    assert.equal(two.stdout, "main :: () {\n  x := 1;\n}\n");
    const broken = await runJaifmt(process.env.JAIFMT!, "main :: () {", dir);
    assert.equal(broken.ok, false);
    assert.match(broken.stderr, /unbalanced|bracket|\{/);
  });
});

describe("jaifmt --is-ignored", { skip: !process.env.JAIFMT && "set JAIFMT to a jaifmt executable" }, () => {
  it("reports the files the config's ignore list covers", async () => {
    const { mkdirSync, mkdtempSync, realpathSync, writeFileSync } = await import("node:fs");
    const { tmpdir } = await import("node:os");
    const dir = realpathSync(mkdtempSync(path.join(tmpdir(), "jaifmt-ignore-")));
    mkdirSync(path.join(dir, "gen", "deep"), { recursive: true });
    writeFileSync(path.join(dir, "jacket.toml"), '[format]\nignore = ["gen/*.jai", "vendor"]\n');
    assert.equal(await isIgnored(process.env.JAIFMT!, path.join(dir, "gen", "a.jai")), true);
    assert.equal(await isIgnored(process.env.JAIFMT!, path.join(dir, "vendor", "x", "a.jai")), true);
    assert.equal(await isIgnored(process.env.JAIFMT!, path.join(dir, "gen", "deep", "a.jai")), false);
    assert.equal(await isIgnored(process.env.JAIFMT!, path.join(dir, "src", "a.jai")), false);
  });

  it("is false when the executable does not exist", async () => {
    assert.equal(await isIgnored(path.join(path.sep, "no", "such", "jaifmt"), path.join(path.sep, "tmp", "a.jai")), false);
  });
});
