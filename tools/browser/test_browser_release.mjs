import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { inspectAssets, checkRelease } from "./check_browser_release.mjs";

const wasm = Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]);
const formatter = Buffer.from([0, 97, 115, 109, 1, 0, 0, 0, 0, 0]);

async function fixture() {
  const temporary = await mkdtemp(path.join(tmpdir(), "jai-own-release-test-"));
  const directory = path.join(temporary, "nested base", "jai", "a".repeat(40));
  await mkdir(directory, { recursive: true });
  await writeFile(path.join(directory, "README.md"), "# own fixture\n");
  await writeFile(path.join(directory, "jaifmt-playground.jai"), "main :: () {}\n");
  await writeFile(path.join(directory, "engine.mjs"), 'export async function createEngine(bytes) { const module = await WebAssembly.compile(bytes); const instance = await WebAssembly.instantiate(module, {}); return { play() { return instance.exports.missing(); } }; }');
  await writeFile(path.join(directory, "jai_wasm.wasm"), wasm);
  await writeFile(path.join(directory, "webgpu_host.mjs"), 'import { STRUCTS } from "./webgpu_bindings.generated.mjs";\nexport function createWebGPUHost() { return { functions: { STRUCTS } }; }\n');
  await writeFile(path.join(directory, "webgpu_bindings.generated.mjs"), "export const STRUCTS = {};\n");
  await writeFile(path.join(directory, "jaifmt.wasm"), formatter);
  await mkdir(path.join(directory, "walk", "nested"), { recursive: true });
  await writeFile(path.join(directory, "walk", "main.jai"), '#load "nested/part.jai";\n');
  await writeFile(path.join(directory, "walk", "nested", "part.jai"), "main :: () {}\n");
  await writeFile(path.join(directory, "walk", "jacket.toml"), "[format]\n");
  await mkdir(path.join(directory, "textured-cube", "assets"), { recursive: true });
  await writeFile(path.join(directory, "textured-cube", "main.jai"), "main :: () {}\n");
  await writeFile(path.join(directory, "textured-cube", "jacket.toml"), "[format]\n");
  await writeFile(path.join(directory, "textured-cube", "assets", "a.png"), Buffer.from([137, 80, 78, 71]));
  const examples = [
    { name: "walk", title: "Walk", summary: "Loads a nested file.", main: "main.jai", files: ["main.jai", "jacket.toml", "nested/part.jai"], assets: [] },
    { name: "textured-cube", title: "Cube", summary: "Loads an asset.", main: "main.jai", files: ["main.jai", "jacket.toml"], assets: ["assets/a.png"] },
  ];
  await writeFile(path.join(directory, "examples.json"), JSON.stringify({ schema_version: 2, examples }));
  const metadata = { schema_version: 1, commit: "a".repeat(40), toolchain: "nightly-2026-08-29", wasm_sha256: createHash("sha256").update(wasm).digest("hex"),
    jaifmt_wasm_sha256: createHash("sha256").update(formatter).digest("hex") };
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

test("the examples index must list exactly the staged example files, and no other folder is allowed", async () => {
  const { temporary, directory } = await fixture();
  try {
    await writeFile(path.join(directory, "walk", "extra.jai"), "x :: 1;\n");
    await assert.rejects(inspectAssets(directory), /list exactly the files/);
    await rm(path.join(directory, "walk", "extra.jai"));
    await writeFile(path.join(directory, "textured-cube", "assets", "b.png"), Buffer.from([137]));
    await assert.rejects(inspectAssets(directory), /list exactly the files/);
    await rm(path.join(directory, "textured-cube", "assets", "b.png"));
    await rm(path.join(directory, "walk", "jacket.toml"));
    await assert.rejects(inspectAssets(directory), /list exactly the files/);
    await writeFile(path.join(directory, "walk", "jacket.toml"), "[format]\n");
    await mkdir(path.join(directory, "ui"));
    await writeFile(path.join(directory, "ui", "index.mjs"), "");
    await assert.rejects(inspectAssets(directory), /Unexpected bundle folder/);
  } finally { await rm(temporary, { recursive: true }); }
});

test("a header-only module cannot pass the actual staged execution gate", async () => {
  const { temporary, directory } = await fixture();
  try { await assert.rejects(checkRelease(directory, { stdlib: false }), /missing/); }
  finally { await rm(temporary, { recursive: true }); }
});
