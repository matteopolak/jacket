import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import * as path from "node:path";
import { describe, it } from "node:test";

const root = path.join(__dirname, "..", "..", "..");
const manifest = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8"));
const id = `${manifest.publisher}.${manifest.name}`;
// The IDs the extension had before, which src/extension.ts looks for on purpose.
const oldIds = ["matteopolak.jai", "matteopolak.jai-toolchain"];

function idsIn(text: string): string[] {
  return [...text.matchAll(new RegExp(`${manifest.publisher}\\.[A-Za-z0-9-]+`, "g"))].map((m) => m[0]);
}

describe("extension ID", () => {
  it("is the one the manifest's default formatter names", () => {
    assert.equal(manifest.contributes.configurationDefaults["[jai]"]["editor.defaultFormatter"], id);
  });

  it("is the only one the sources name, apart from the old ID", () => {
    const src = path.join(root, "src");
    for (const file of readdirSync(src).filter((name) => name.endsWith(".ts"))) {
      for (const found of idsIn(readFileSync(path.join(src, file), "utf8"))) {
        assert.ok(found === id || oldIds.includes(found), `${file} names ${found}, not ${id}`);
      }
    }
  });

  it("is not one of the old IDs, and extension.ts looks for each of them", () => {
    assert.ok(!oldIds.includes(id));
    const source = readFileSync(path.join(root, "src", "extension.ts"), "utf8");
    for (const old of oldIds) assert.ok(source.includes(`"${old}"`), `extension.ts does not look for ${old}`);
  });
});
