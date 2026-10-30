// test: skip_on(.WASM, reason = "File_Utilities has no wasm backend")
Basic :: #import "Basic";
Files :: #import "File_Utilities";

consume :: (value: Basic.Apollo_Time) {
    assert(type_of(value) == Basic.Apollo_Time);
}

main :: () {
    modification, _, _ := Files.file_modtime_and_size(".");
    consume(modification);
}

#import "Basic";
