// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target")
#import "Basic";
WT :: #import "Window_Type";

main :: () {
    window: WT.Window_Type = WT.INVALID_WINDOW;
    print("%\n", window);
}
