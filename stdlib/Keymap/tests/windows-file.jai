// test: skip_on(.WASM, reason = "needs the native SDL library, which has no wasm build")
K :: #import "Keymap";

main :: () {
    map: K.Keymap;
    K.reset_keymap(*map);
}
