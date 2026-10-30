// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)")
#import "Basic";
#import "Sound_Player";

main :: () {
    // Direct advancement also works before init and without recording samples.
    // Private extrema/counters have no public observation API.
    for 0..129 advance_history_cursor();
    print("ok\n");
}
