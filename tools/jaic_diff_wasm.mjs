#!/usr/bin/env node
// Wasm backend for tools/jaic-diff.py: a long-lived process that runs programs through the browser engine
// (crates/jai-wasm) and answers one JSON line per request line on stdin.
//   request:  {"root": "/abs/dir", "main": "rel/path.jai", "budget": 0}
//   reply:    {"exitCode": n|null, "stdout": "...", "stderr": "...", "diagnostics": [...], "crash": "..."?}
// Every text file under `root` becomes a workspace file, as check_playground_stdlib.mjs does for tests/stdlib.
// A compiler panic traps the instance, so the engine is rebuilt after one; it is also rebuilt every
// RECYCLE runs because wasm memory only grows.
import { readFile, readdir, stat } from "node:fs/promises";
import path from "node:path";
import readline from "node:readline";
import { pathToFileURL } from "node:url";

const bundle = path.resolve(process.argv[2] ?? "");
if (!process.argv[2]) throw new Error("usage: node tools/jaic_diff_wasm.mjs <bundle-dir>");
const { createEngine } = await import(pathToFileURL(path.join(bundle, "engine.mjs")).href);
const wasmBytes = await readFile(path.join(bundle, "jai_wasm.wasm"));
const RECYCLE = 20;
const MAX_FILES = 2000;
const MAX_BYTES = 32 * 1024 * 1024;

let engine = null;
let runs = 0;

async function collect(root) {
  const files = {};
  let count = 0, bytes = 0;
  async function walk(dir) {
    for (const entry of await readdir(dir, { withFileTypes: true })) {
      if (entry.name.startsWith(".")) continue;
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) await walk(full);
      else if (entry.isFile()) {
        const size = (await stat(full)).size;
        if (++count > MAX_FILES || (bytes += size) > MAX_BYTES) throw new Error(`workspace ${root} is too large for the wasm backend`);
        // The engine takes text files only; a binary file is left out (the program then fails to open it).
        let text;
        try { text = new TextDecoder("utf-8", { fatal: true }).decode(await readFile(full)); } catch { continue; }
        files[path.relative(root, full).replaceAll("\\", "/")] = text;
      }
    }
  }
  await walk(root);
  return files;
}

async function handle(request) {
  const files = await collect(request.root);
  if (!engine || runs >= RECYCLE) { engine = await createEngine(wasmBytes); runs = 0; }
  runs++;
  try {
    const result = engine.play(files, request.main, request.budget ? { budget: request.budget } : {});
    return { exitCode: result.exitCode, stdout: result.stdout, stderr: result.stderr, diagnostics: result.diagnostics };
  } catch (error) {
    engine = null;
    return { exitCode: null, stdout: "", stderr: "", diagnostics: [], crash: String(error.message ?? error) };
  }
}

const lines = readline.createInterface({ input: process.stdin });
for await (const line of lines) {
  if (!line.trim()) continue;
  let reply;
  try { reply = await handle(JSON.parse(line)); }
  catch (error) { reply = { exitCode: null, stdout: "", stderr: "", diagnostics: [], crash: `driver: ${error.message ?? error}` }; }
  process.stdout.write(JSON.stringify(reply) + "\n");
}
