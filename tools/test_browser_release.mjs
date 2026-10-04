import assert from "node:assert/strict";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { inspectAssets, checkRelease } from "./check_browser_release.mjs";

async function fixture() {
  const temporary = await mkdtemp(path.join(tmpdir(), "jai-own-release-test-"));
  const directory = path.join(temporary, "nested base", "jai", "a".repeat(40));
  await mkdir(directory, { recursive: true });
  await writeFile(path.join(directory, "index.html"), '<link rel="stylesheet" href="./style.css"><script type="module" src="./editor.mjs"></script>');
  await writeFile(path.join(directory, "style.css"), "body {}");
  await writeFile(path.join(directory, "editor.mjs"), 'import "./workspace.mjs"; new Worker(new URL("./worker.mjs", import.meta.url));');
  await writeFile(path.join(directory, "workspace.mjs"), "// own closed fixture\n");
  await writeFile(path.join(directory, "worker.mjs"), 'import "./engine.mjs"; fetch(new URL("./jai_wasm.wasm", import.meta.url));');
  await writeFile(path.join(directory, "engine.mjs"), 'export async function createEngine(bytes) { const module = await WebAssembly.compile(bytes); const instance = await WebAssembly.instantiate(module, {}); return { play() { return instance.exports.missing(); } }; }');
  await writeFile(path.join(directory, "jai_wasm.wasm"), Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]));
  await writeFile(path.join(directory, "release.json"), JSON.stringify({ schema_version: 1, commit: "a".repeat(40) }));
  return { temporary, directory };
}

test("authored dependency closure remains valid at a nested base containing spaces", async () => {
  const { temporary, directory } = await fixture();
  try { assert.equal((await inspectAssets(directory)).commit, "a".repeat(40)); }
  finally { await rm(temporary, { recursive: true }); }
});

test("missing assets, traversal and CDN imports are rejected", async () => {
  const { temporary, directory } = await fixture();
  try {
    for (const dependency of ["./missing.mjs", "../outside.mjs", "https://cdn.example.invalid/editor.mjs"]) {
      await writeFile(path.join(directory, "editor.mjs"), `import ${JSON.stringify(dependency)};`);
      await assert.rejects(inspectAssets(directory), /Missing|escapes|non-relative/);
    }
  } finally { await rm(temporary, { recursive: true }); }
});

test("a header-only module cannot pass the actual staged execution gate", async () => {
  const { temporary, directory } = await fixture();
  try { await assert.rejects(checkRelease(directory), /missing/); }
  finally { await rm(temporary, { recursive: true }); }
});
