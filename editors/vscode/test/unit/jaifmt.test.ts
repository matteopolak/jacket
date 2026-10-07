import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import * as path from "node:path";
import { describe, it } from "node:test";
import { findConfig, globToRegExp, ignoreGlobs, isIgnored, minimalReplacement, runJaifmt } from "../../src/jaifmt";

function repoFile(rel: string): string {
  return path.join(path.sep, "repo", ...rel.split("/"));
}

function apply(text: string, change: ReturnType<typeof minimalReplacement>): string {
  return change ? text.slice(0, change.start) + change.text + text.slice(change.end) : text;
}

describe("jaifmt.toml", () => {
  it("is found in the file's directory or above", () => {
    const files = new Set([path.join("/repo", "jaifmt.toml")]);
    assert.equal(findConfig("/repo/src/deep", (f) => files.has(f)), path.join("/repo", "jaifmt.toml"));
    assert.equal(findConfig("/elsewhere", (f) => files.has(f)), undefined);
  });

  it("ignore globs are read from the one-line array", () => {
    const toml = '# settings\nindent_width = 4\nignore = ["tests/corpus/**", "gen/*.jai"]  # why\n';
    assert.deepEqual(ignoreGlobs(toml), ["tests/corpus/**", "gen/*.jai"]);
    assert.deepEqual(ignoreGlobs("indent_width = 2\n"), []);
  });

  it("globs follow jaifmt: `*` within a component, `**` across, directories cover their files", () => {
    assert.ok(globToRegExp("gen/*.jai").test("gen/a.jai"));
    assert.ok(!globToRegExp("gen/*.jai").test("gen/sub/a.jai"));
    assert.ok(globToRegExp("tests/corpus/**").test("tests/corpus/x/y.jai"));
    assert.ok(globToRegExp("**/fixtures").test("a/b/fixtures/c.jai"));
    assert.ok(globToRegExp("fuzz").test("fuzz/seeds/1.jai"));
    assert.ok(!globToRegExp("fuzz").test("fuzzy.jai"));
    assert.ok(globToRegExp("a?.jai").test("ab.jai"));
  });

  it("matches files relative to the config's directory", () => {
    const config = path.join(path.sep, "repo", "jaifmt.toml");
    assert.ok(isIgnored(config, ["tests/corpus/**"], repoFile("tests/corpus/ok/a.jai")));
    assert.ok(!isIgnored(config, ["tests/corpus/**"], repoFile("tests/other/a.jai")));
    assert.ok(!isIgnored(config, ["**"], path.join(path.sep, "elsewhere", "a.jai")));
  });

  it("the repository's own ignores apply", () => {
    // The root jaifmt.toml leaves the formatter's golden inputs alone.
    const config = path.join(__dirname, "..", "..", "..", "..", "..", "jaifmt.toml");
    const globs = ignoreGlobs(readFileSync(config, "utf8"));
    const golden = path.join(path.dirname(config), "stdlib", "Extensions", "Jai_Format", "tests", "cases", "spacing.in.jai");
    assert.ok(isIgnored(config, globs, golden));
    assert.ok(!isIgnored(config, globs, path.join(path.dirname(config), "examples", "tour", "main.jai")));
  });
});

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
