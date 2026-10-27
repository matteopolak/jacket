// test: skip_on(.WASM) compile_fail
#import "Sound_Player";

main :: () {
    pointer := *sound_mutex;
}
