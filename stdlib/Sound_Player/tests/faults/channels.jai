// test: skip_on(.WASM) should_fail("audio sample input/channel range invalid")
A :: #import "Sound_Player";

main :: () {
    sample: A.Audio_Sample_Base;
    A.plus_equals(*sample, *sample, 9);
}
