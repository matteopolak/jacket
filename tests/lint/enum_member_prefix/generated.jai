// Independently generated with the public libclang C API.
// Generated bindings keep C's member names: `enum_member_prefix` skips this file.
#import "Basic";

Mode :: enum u32 {
    MODE_READ :: 1;
    MODE_WRITE :: 2;
}

main :: () {
    m := Mode.MODE_READ;
    assert(m != .MODE_WRITE);
}
