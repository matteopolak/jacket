import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { inspectAssets, checkRelease } from "./check_browser_release.mjs";

const wasm = Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]);

async function fixture() {
  const temporary = await mkdtemp(path.join(tmpdir(), "jai-own-release-test-"));
  const directory = path.join(temporary, "nested base", "jai", "a".repeat(40));
  await mkdir(directory, { recursive: true });
  await writeFile(path.join(directory, "README.md"), "# own fixture\n");
  await writeFile(path.join(directory, "jaifmt-playground.jai"), "main :: () {}\n");
  await writeFile(path.join(directory, "engine.mjs"), 'export async function createEngine(bytes) { const module = await WebAssembly.compile(bytes); const instance = await WebAssembly.instantiate(module, {}); return { play() { return instance.exports.missing(); } }; }');
  await writeFile(path.join(directory, "jai_wasm.wasm"), wasm);
  const metadata = { schema_version: 1, commit: "a".repeat(40), toolchain: "nightly-2026-08-29", wasm_sha256: createHash("sha256").update(wasm).digest("hex") };
  await writeFile(path.join(directory, "build-metadata.json"), JSON.stringify(metadata));
  return { temporary, directory };
}

test("the exact bundle inventory is accepted at a nested base containing spaces", async () => {
  const { temporary, directory } = await fixture();
  try { assert.equal((await inspectAssets(directory)).commit, "a".repeat(40)); }
  finally { await rm(temporary, { recursive: true }); }
});

test("UI leftovers, a stale module digest and engine imports are rejected", async () => {
  const { temporary, directory } = await fixture();
  try {
    await writeFile(path.join(directory, "index.html"), "<p>stale UI</p>");
    await assert.rejects(inspectAssets(directory), /exactly/);
    await rm(path.join(directory, "index.html"));
    await writeFile(path.join(directory, "jai_wasm.wasm"), Buffer.from([0, 97, 115, 109, 1, 0, 0, 0, 0]));
    await assert.rejects(inspectAssets(directory), /describe the staged module/);
    await writeFile(path.join(directory, "jai_wasm.wasm"), wasm);
    await writeFile(path.join(directory, "engine.mjs"), 'import "./worker.mjs"; export const createEngine = () => {};');
    await assert.rejects(inspectAssets(directory), /self-contained/);
  } finally { await rm(temporary, { recursive: true }); }
});

test("a header-only module cannot pass the actual staged execution gate", async () => {
  const { temporary, directory } = await fixture();
  try { await assert.rejects(checkRelease(directory, { stdlib: false }), /missing/); }
  finally { await rm(temporary, { recursive: true }); }
});
