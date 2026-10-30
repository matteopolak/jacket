// test: skip_on(.WASM, reason = "needs OpenGL, which has no wasm target") compile_fail
#import "Simp";

main :: () {
    pack_bitmap(null);
}
