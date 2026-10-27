// test: skip_on(.WASM) should_fail("em requires a font")
// Metrics require an actual canonical font pointer.
G :: #import "GetRect";

main :: () {
    G.em(1.0, null);
}
