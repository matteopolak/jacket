// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") should_fail("audio sample input/channel range invalid")
A :: #import "Sound_Player";

main :: () {
    sample: A.Audio_Sample_Base;
    A.plus_equals(*sample, *sample, 9);
}
