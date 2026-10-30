// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target") should_fail("texture loader callback required")
// Texture creation requires the caller's actual resource loader.
#import "GetRect";

main :: () {
    require_texture("caller-provided pixels");
}
