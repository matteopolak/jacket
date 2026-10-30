// test: skip_on(.WASM, reason = "needs a native window, and Window_Type has no wasm target") should_fail("em requires a font")
// Metrics require an actual canonical font pointer.
G :: #import "GetRect";

main :: () {
    G.em(1.0, null);
}
