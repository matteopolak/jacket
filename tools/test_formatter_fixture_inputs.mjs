import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { goldenCaseNames, isAppleDoubleMetadata, optionalConfig, readUtf8 } from "./formatter_fixture_inputs.mjs";

function metadata() {
  const bytes = Buffer.alloc(39);
  Buffer.from([0, 5, 0x16, 7, 0, 2, 0, 0]).copy(bytes);
  bytes.writeUInt16BE(1, 24);
  bytes.writeUInt32BE(2, 26);
  bytes.writeUInt32BE(38, 30);
  bytes.writeUInt32BE(1, 34);
  bytes[38] = 0xff;
  return bytes;
}

test("only named complete in-bounds AppleDouble metadata is excluded", () => {
  const valid = metadata();
  assert.equal(isAppleDoubleMetadata("._case.in.jai", valid), true);
  assert.equal(isAppleDoubleMetadata("case.in.jai", valid), false);
  assert.equal(isAppleDoubleMetadata("._case.in.jai", Buffer.from("value :: 1;")), false);
  for (let end = 0; end < valid.length; end++) {
    assert.equal(isAppleDoubleMetadata("._case.in.jai", valid.subarray(0, end)), false);
  }
  for (const [position, value] of [[0, 1], [4, 1], [30, 0], [30, 40], [34, 2], [34, 0xffffffff]]) {
    const bad = Buffer.from(valid);
    bad.writeUInt32BE(value, position);
    assert.equal(isAppleDoubleMetadata("._case.in.jai", bad), false);
  }
  const missingTable = Buffer.from(valid);
  missingTable.writeUInt16BE(2, 24);
  assert.equal(isAppleDoubleMetadata("._case.in.jai", missingTable), false);
});

test("transport sidecars do not change repository golden case coverage", () => {
  const cases = fileURLToPath(new URL("../stdlib/Extensions/Jai_Format/tests/cases/", import.meta.url));
  const names = goldenCaseNames(cases);
  assert.ok(names.length > 0);
  const root = mkdtempSync(path.join(tmpdir(), "formatter-goldens-"));
  try {
    for (const name of names) {
      readUtf8(path.join(cases, `${name}.in.jai`));
      readUtf8(path.join(cases, `${name}.out.jai`));
      optionalConfig(path.join(cases, `${name}.toml`));
      writeFileSync(path.join(root, `${name}.in.jai`), readFileSync(path.join(cases, `${name}.in.jai`)));
      writeFileSync(path.join(root, `._${name}.in.jai`), metadata());
    }
    assert.deepEqual(goldenCaseNames(root), names);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("discovery retains all real cases, including sidecar-named and invalid text", () => {
  const root = mkdtempSync(path.join(tmpdir(), "formatter-inputs-"));
  try {
    writeFileSync(path.join(root, "ordinary.in.jai"), "main :: () {}\n");
    writeFileSync(path.join(root, "._legitimate.in.jai"), "value :: 1;\n");
    writeFileSync(path.join(root, "._metadata.in.jai"), metadata());
    const malformed = metadata().subarray(0, 38);
    malformed[8] = 0xff; // Invalid UTF-8, plus an entry whose payload is missing.
    writeFileSync(path.join(root, "._malformed.in.jai"), malformed);
    writeFileSync(path.join(root, "invalid.in.jai"), Buffer.from([0xff]));
    assert.deepEqual(goldenCaseNames(root), ["._legitimate", "._malformed", "invalid", "ordinary"]);
    assert.throws(() => readUtf8(path.join(root, "invalid.in.jai")));
    assert.throws(() => readUtf8(path.join(root, "._malformed.in.jai")));
    assert.equal(optionalConfig(path.join(root, "absent.toml")), undefined);
    writeFileSync(path.join(root, "invalid.toml"), Buffer.from([0xff]));
    assert.throws(() => optionalConfig(path.join(root, "invalid.toml")));
    assert.throws(() => optionalConfig(root));
    writeFileSync(path.join(root, "bom.jai"), "\ufeffvalue :: 1;\n");
    assert.equal(readUtf8(path.join(root, "bom.jai")), "\ufeffvalue :: 1;\n");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
