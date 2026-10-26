import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { Listed } from "../../src/testEvents";
import { buildTree, testId } from "../../src/testTree";

function listed(name: string, kind: string, file: string, line: number): Listed {
  return { name, kind, file, line, notes: [] };
}

describe("the test tree", () => {
  it("groups tests by file, in path and source order", () => {
    const tree = buildTree([
      listed("late", "test", "/p/tests/b.jai", 30),
      listed("early", "test", "/p/tests/b.jai", 4),
      listed("only", "test", "/p/tests/a.jai", 9),
    ]);
    assert.deepEqual(
      tree.map((f) => [f.file, f.tests.map((t) => t.name)]),
      [
        ["/p/tests/a.jai", ["only"]],
        ["/p/tests/b.jai", ["early", "late"]],
      ],
    );
  });

  it("puts the cases of a test under it, whatever the case is called", () => {
    const tree = buildTree([
      listed("paths", "test", "/p/tests/a.jai", 24),
      listed("paths/a/b", "subtest", "/p/tests/a.jai", 24),
      listed("paths/2", "subtest", "/p/tests/a.jai", 24),
      listed("other", "test", "/p/tests/a.jai", 40),
    ]);
    const [paths, other] = tree[0].tests;
    assert.deepEqual(
      paths.children.map((c) => [c.name, c.label]),
      [
        ["paths/a/b", "a/b"],
        ["paths/2", "2"],
      ],
    );
    assert.deepEqual(other.children, []);
  });

  it("keeps golden, compile-fail and doc tests, and leaves out benchmarks", () => {
    const tree = buildTree([
      listed("golden-args", "golden", "/p/tests/golden-args.jai", 1),
      listed("bad", "compile_fail", "/p/tests/compile_fail/bad.jai", 1),
      listed("doc: Testing.run#1", "doc", "/p/module.jai", 10),
      listed("speed", "bench", "/p/tests/a.jai", 3),
    ]);
    assert.deepEqual(
      tree.flatMap((f) => f.tests.map((t) => t.kind)),
      ["doc", "compile_fail", "golden"],
    );
  });

  it("drops a case whose test was not listed", () => {
    assert.deepEqual(buildTree([listed("lost/1", "subtest", "/p/tests/a.jai", 5)]), []);
  });

  it("names items by file and test", () => {
    assert.equal(testId("/p/tests/a.jai", "adds"), "/p/tests/a.jai#adds");
  });
});
