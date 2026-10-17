#import "Basic";
V :: #import "developer-tools-value-emission-generated";

main :: () {
    assert(V.minimum == -9223372036854775807 - 1);
    assert(V.maximum == 0xffffffffffffffff);
    assert(type_of(V.maximum) == u64);
    assert(V.unsigned_byte == 255 && V.signed_byte == -1);
    assert(V.converted_unsigned == 255 && type_of(V.converted_unsigned) == u8);
    assert(V.converted_signed == -1 && type_of(V.converted_signed) == s8);
    assert(V.converted_bool && type_of(V.converted_bool) == bool);
    floating: float32 = V.converted_float;

    assert((cast(*u32) *floating).* == 0x3dcccccd);
    negative_zero: float64 = V.negative_zero;

    assert((cast(*u64) *negative_zero).* == 0x8000000000000000);
    bytes := u8.[0, 1, 10, 13, 34, 92, 255, 65, 55];

    assert(V.bytes.count == bytes.count);
    for byte, index: bytes assert(V.bytes[index] == byte);

    assert(V.mode == .VALUE && V.identifier == 42);
    assert(V.precedence == 21 && V.left_shift == 48 && V.right_shift == 10 && V.zero_shift == 19);
    assert(V.complement == 0xfffffffffffffffa && V.native_size == 4);
    print("ok\n");
}
