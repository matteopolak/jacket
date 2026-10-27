// test: skip_on(.WASM) should_fail("audio magic read outside input")
A :: #import "Sound_Player";

main :: () {
    A.get_magic4("abc", 0);
}
