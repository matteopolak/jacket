#!/usr/bin/env node
// Bounded actual-module replay; does not replace the full compiler/tour gate.
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";
import { checkScriptingIo } from "./scripting_io_wasm.mjs";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_scripting_io_wasm.mjs <jai_wasm.wasm>");
const bytes = await readFile(binary);
await checkScriptingIo(await createEngine(bytes), options => createEngine(bytes, options));
