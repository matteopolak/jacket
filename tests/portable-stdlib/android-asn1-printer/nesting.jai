// Explicit traversal-stack safety and exact four-space indentation.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    bytes: [1202] u8;
    cursor := bytes.count - 2;
    bytes[cursor] = 5;
    bytes[cursor + 1] = 0;

    for 0..299 {
        count := bytes.count - cursor;

        cursor -= 4;
        bytes[cursor] = 0x30;
        bytes[cursor + 1] = 0x82;
        bytes[cursor + 2] = cast(u8) (count >> 8);
        bytes[cursor + 3] = cast,truncate(u8) count;
    }

    assert(cursor == 0);
    A.asn1_print(.{ bytes.count, bytes.data });
}
