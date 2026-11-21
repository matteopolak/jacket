#import "Basic";

// The same integers seen through narrower and wider pointers: the byte order shows.
main :: () {
    value: u32 = 0x01020304;
    bytes := cast(*u8) *value;

    print("% % % %\n", bytes[0], bytes[1], bytes[2], bytes[3]);

    wide: u64 = 0x0102030405060708;
    half := cast(*u32) *wide;

    print("%\n", half.*);

    bits: float32 = 1.5;
    as_integer := cast(*u32) *bits;

    print("%\n", as_integer.*);
}
