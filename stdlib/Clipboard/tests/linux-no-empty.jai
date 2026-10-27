// test: only(.LINUX) skip_on(.WASM) compile_fail
C :: #import "Clipboard";

main :: () {
    C.os_clipboard_empty();
}
