import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { collectPlayFiles, playRequestSize } from "./playground_request.mjs";

async function workspace(run) {
  const base = await mkdtemp(path.join(tmpdir(), "jai-play-request-"));
  const put = async (name, text) => {
    const full = path.join(base, name);
    await mkdir(path.dirname(full), { recursive: true });
    await writeFile(full, text);
  };
  try { await run(base, put); } finally { await rm(base, { recursive: true, force: true }); }
}

test("module request preserves every source and test fixture without audit payload", async () => {
  await workspace(async (base, put) => {
    await put("Module/module.jai", "public_value :: 42;");
    await put("Module/tests/main.jai", '#load "../../internal/helper.jai"; main :: () {}');
    await put("internal/helper.jai", "helper :: 7;");
    await put("Other/tests/data/fixture.json", '{"value":19}');
    await put("Other/tests/data/input.txt", "fixture");
    await put("Module/tests/README.md", "test fixture documentation");
    await put("Module/coverage.json", " ".repeat(16 * 1024 * 1024));
    await put("Module/inventory.json", '{"audit":true}');
    const legacy = {};
    await collectPlayFiles(base, base, legacy);
    const files = {};
    await collectPlayFiles(base, base, files, { moduleWorkspace: true });
    assert.deepEqual(Object.keys(files).filter(n => n.endsWith(".jai")),
      Object.keys(legacy).filter(n => n.endsWith(".jai")), "all sources, including cross-directory loads, remain");
    assert.equal(files["Other/tests/data/fixture.json"], '{"value":19}');
    assert.equal(files["Other/tests/data/input.txt"], "fixture");
    assert.equal(files["Module/tests/README.md"], "test fixture documentation");
    assert.equal(files["Module/coverage.json"], undefined);
    assert.equal(files["Module/inventory.json"], undefined);
    assert(playRequestSize(legacy, "Module/tests/main.jai").totalBytes > 16 * 1024 * 1024);
    assert(playRequestSize(files, "Module/tests/main.jai").totalBytes < 16 * 1024 * 1024);
  });
});

test("independent regression workspace retains all existing text input classes", async () => {
  await workspace(async (base, put) => {
    for (const name of ["main.jai", "data.json", "input.txt", "README.md"]) await put(name, name);
    const files = {};
    await collectPlayFiles(base, base, files);
    assert.deepEqual(Object.keys(files).sort(), ["README.md", "data.json", "input.txt", "main.jai"]);
  });
});

test("request size counts actual UTF-8 file paths, contents and main once", () => {
  const files = { "🧭.jai": "é", "second.jai": "abc" };
  const actual = Object.entries(files).reduce((n, [p, s]) => n + new TextEncoder().encode(p).length + new TextEncoder().encode(s).length, new TextEncoder().encode("🧭.jai").length);
  assert.deepEqual(playRequestSize(files, "🧭.jai"), {
    fileCount: 2, pathBytes: 8 + 10, sourceBytes: 2 + 3, mainBytes: 8, totalBytes: actual,
  });
});

test("oversized genuine source stays oversized rather than weakening the boundary", () => {
  const files = { "main.jai": " ".repeat(16 * 1024 * 1024) };
  assert(playRequestSize(files, "main.jai").totalBytes > 16 * 1024 * 1024);
  assert.equal(files["main.jai"].length, 16 * 1024 * 1024);
});

test("AppleDouble filesystem sidecars are not workspace source or fixture files", async () => {
  await workspace(async (base, put) => {
    await put("main.jai", "main :: () {}");
    await put("._main.jai", "filesystem metadata");
    await put("Module/tests/._fixture.json", "filesystem metadata");
    await put("Module/tests/.settings.json", '{"real_fixture":true}');
    const files = {};
    await collectPlayFiles(base, base, files, { moduleWorkspace: true });
    assert.equal(files["._main.jai"], undefined);
    assert.equal(files["Module/tests/._fixture.json"], undefined);
    assert.equal(files["main.jai"], "main :: () {}");
    assert.equal(files["Module/tests/.settings.json"], '{"real_fixture":true}');
  });
});


test("module entry and supplied library keep their canonical relative layout", async () => {
  await workspace(async (base, put) => {
    await put("stdlib/prelude/module.jai", "prelude source");
    await put("stdlib/internal/helper.jai", "private_answer :: 7;");
    await put("stdlib/Example/tests/main.jai", "#import \"internal/helper\";");
    await put("stdlib/Other/tests/data.json", "fixture");
    const { buildPlayRequest } = await import("./playground_request.mjs");
    const request = await buildPlayRequest(base, { family: "modules", path: "stdlib/Example/tests/main.jai" });
    assert.equal(request.libraryRoot, "stdlib");
    assert.equal(request.main, "stdlib/Example/tests/main.jai");
    assert.equal(request.files["stdlib/internal/helper.jai"], "private_answer :: 7;");
    assert.equal(request.files["stdlib/Other/tests/data.json"], "fixture");
    assert.equal(request.request.libraryRootBytes, 6);
    assert.equal(request.request.totalBytes, playRequestSize(request.files, request.main).totalBytes + 6);
  });
});
