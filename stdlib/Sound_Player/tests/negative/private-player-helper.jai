// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") compile_fail
#import "Sound_Player";

main :: () {
    lock_player();
}
