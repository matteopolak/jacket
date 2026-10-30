// test: skip_on(.WASM, reason = "File_Utilities has no wasm backend")
#import "Basic";
#import "File_Utilities";

main :: () {
    modification, size, exists := file_modtime_and_size(".");
    milliseconds, converted := to_milliseconds(modification);
    if exists assert(size >= 0);
    if converted assert(milliseconds >= 0);
}
