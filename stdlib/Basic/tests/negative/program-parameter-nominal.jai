// test: skip_on(.WASM, reason = "File_Utilities has no wasm backend") compile_fail
Basic :: #import "Basic"; // jailint: allow(unused_import) the declaration is part of what the test compiles
Files :: #import "File_Utilities";

Unrelated :: struct {
    low: u64;
    high: s64;
}

consume :: (value: Unrelated) {}

main :: () {
    modification, _, _ := Files.file_modtime_and_size(".");
    consume(modification);
}
