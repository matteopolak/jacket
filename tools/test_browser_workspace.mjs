import assert from "node:assert/strict";
import { test } from "node:test";
import { SourcePath, Workspace } from "../web/scripting-runtime/workspace.mjs";

test("parsed file identities reject escaping paths and unrepresentable names", () => {
  for (const name of ["../secret.jai", "/root.jai", "a/../../secret.jai", "x\\y.jai", "x\0.jai", "", "\ud800", "x".repeat(4097)]) {
    assert.throws(() => SourcePath.parse(name));
  }
  assert.throws(() => new SourcePath("../secret.jai", SourcePath));
  assert.equal(SourcePath.parse("lib/./nested/../value.jai").name, "lib/value.jai");
  assert.equal(SourcePath.parse("資料/値.jai").name, "資料/値.jai");
});

test("canonical aliases cannot overwrite another file or the entry", () => {
  const workspace = new Workspace("main :: () -> int { return 42; }");
  workspace.add("lib/value.jai", "answer :: 42;");
  const before = workspace.snapshot();
  assert.throws(() => workspace.add("lib/./value.jai", "answer :: 0;"));
  assert.throws(() => workspace.add("lib/../main.jai", "main :: () {}"));
  assert.deepEqual(workspace.snapshot(), before);
  workspace.select("main.jai");
  assert.throws(() => workspace.removeSelected());
});

test("a running source snapshot is isolated from later editor changes", () => {
  const workspace = new Workspace('#load "helper.jai"; main :: () -> int { return answer(); }');
  workspace.add("helper.jai", "answer :: () -> int { return 42; }");
  const snapshot = workspace.snapshot();
  workspace.edit("answer :: () -> int { return 9; }");
  workspace.removeSelected();
  assert.equal(snapshot.files["helper.jai"], "answer :: () -> int { return 42; }");
  assert.equal(workspace.selected.path.name, "main.jai");
  assert.deepEqual(workspace.snapshot().files, {});
  assert.throws(() => { snapshot.files["helper.jai"] = "changed"; });
});
