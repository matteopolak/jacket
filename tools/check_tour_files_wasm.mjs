// Execute the private tour directory adapter against the real WASM sandbox.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node tools/check_tour_files_wasm.mjs <jai_wasm.wasm>");
const engine = await createEngine(await readFile(binary));
const provider = await readFile(new URL("../examples/tour/files/sandbox_files.jai", import.meta.url), "utf8");
const source = `#import "Basic";
#import "File";
#load "sandbox_files.jai";
Count :: struct { files: int; bytes: int; }
visit :: (info: *Tour_File_Visit_Info, result: *Count) {
    text, ok := read_entire_file(info.full_name);
    assert(ok);
    defer free(text);
    result.files += 1;
    result.bytes += text.count;
}
main :: () {
    result: Count;
    assert(tour_visit_files("/workspace/data", true, *result, visit));
    assert(result.files == 2 && result.bytes == 8);
    shallow: Count;
    assert(tour_visit_files("/workspace/data", false, *shallow, visit));
    assert(shallow.files == 1 && shallow.bytes == 3);
    assert(!tour_visit_files("/workspace/missing", true, *result, visit));
    assert(!tour_visit_files("/workspace/data/one.txt", true, *result, visit));
    assert(tour_file_exists("/workspace/data/nested/two words.txt"));
    path := "/tmp/private-tour-files.txt";
    assert(write_entire_file(path, "real file bytes"));
    assert(tour_file_exists(path));
    data, ok := read_entire_file(path);
    assert(ok && data == "real file bytes");
    free(data);
    assert(file_delete(path));
    assert(!tour_file_exists(path));
    print("private files ok\\n");
}`;
const result = engine.play({
  "main.jai": source,
  "sandbox_files.jai": provider,
  "data/one.txt": "one",
  "data/nested/two words.txt": "12345",
}, "main.jai");
assert.deepEqual(result.diagnostics, [], result.rendered);
assert.equal(result.exitCode, 0, result.rendered);
assert.equal(result.stdout, "private files ok\n");
console.log("PASS: actual WASM recursive/shallow directory counts, byte reads, missing/non-directory errors, write/read/delete");
