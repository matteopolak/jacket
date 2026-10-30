// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target")
// Canonical dependency reproducer: no numerics types or test fixtures.
WT :: #import "Window_Type";

main :: () {
    window: WT.Window_Type;
    assert(window == WT.INVALID_WINDOW);
}

#scope_file
#import "Basic";
