// test: skip_on(.WASM, reason = "needs OpenGL, which has no wasm target") compile_fail
#import "Simp";

main :: () {
    atlas_create_page(64, 64);
}
