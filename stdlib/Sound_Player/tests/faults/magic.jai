// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") should_fail("audio magic read outside input")
A :: #import "Sound_Player";

main :: () {
    A.get_magic4("abc", 0);
}
