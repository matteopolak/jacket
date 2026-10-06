#import "Basic";

Color :: enum u8 {
    RED;
    GREEN;
}

main :: () {
    big := 70000;
    // Masked or reduced to fit.
    low: u16 = xx (big & 0xFFFF);
    rest: u8 = xx (big % 256);
    // Constants are checked by the compiler.
    five: u8 = xx 5;
    // Widening loses nothing.
    wide: s64 = xx low;
    // Spelled out.
    narrow := cast(u16) big;
    // Integers become enum values with `xx`.
    index := 1;
    color: Color = xx index;
    print("% % % % % %\n", low, rest, five, wide, narrow, color);
}
