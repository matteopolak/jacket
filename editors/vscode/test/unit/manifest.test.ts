import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import * as path from "node:path";
import { describe, it } from "node:test";

const root = path.join(__dirname, "..", "..", "..");
const manifest = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8"));
const id = `${manifest.publisher}.${manifest.name}`;
// The ID the extension had up to 0.4.1, which src/extension.ts looks for on purpose.
const oldId = "matteopolak.jai";

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
        assert.ok(found === id || found === oldId, `${file} names ${found}, not ${id}`);
      }
    }
  });
});
