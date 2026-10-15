// Reviewed ASN.1 public shapes plus independent standard primitive wire vectors.
#import "Basic";
A :: #import "Android/Toolchain";

check_integer :: ($T: Type, expected: T, little: string, big: string) {
    cursor := little;
    value := A.read(T, *cursor);

    assert(value == expected && cursor.count == 0);
    cursor = big;
    value = A.read(T, *cursor, BIG_ENDIAN = true);
    assert(value == expected && cursor.count == 0);
}

main :: () {
    #assert size_of(A.ASN1_Tag) == 3;
    assert(type_info(A.ASN1_Tag).name == "ASN1_Tag");
    assert(A.ASN1_KNOWN_OIDS.count == 1);
    assert(type_info(type_of(A.ASN1_KNOWN_OIDS[0])).name == "ASN1_Object_Identifier");
    assert(A.asn1_oid_equals(A.ASN1_KNOWN_OIDS[0], A.OID_SUN_JKS_ENCRYPTION));
    overlay: A.ASN1_Tag;
    overlay.number = 6;
    assert(overlay.universal_type == .OBJECT_IDENTIFIER);

    for octet: 0..255 {
        if (octet & 31) == 31 continue;
        if octet == 0 || (octet >> 6) == 1 || (octet >> 6) == 3 continue;
        if (octet >> 6) == 0 && ((octet & 31) == 8 || (octet & 31) == 15 || (octet & 31) == 29) continue;

        storage: [1] u8 = .[cast(u8) octet];
        cursor: string = .{ 1, storage.data };
        decoded := A.asn1_read_tag(*cursor);

        assert(decoded.number == (octet & 31) && cast(u8) decoded.class == (octet >> 6));
        assert(cast(u8) decoded.length_value == ((octet >> 5) & 1) && cursor.count == 0);
    }

    check_integer(u8, 0xab, "\xab", "\xab");
    check_integer(u16, 0xabcd, "\xcd\xab", "\xab\xcd");
    check_integer(u32, 0x89abcdef, "\xef\xcd\xab\x89", "\x89\xab\xcd\xef");
    check_integer(u64, 0x0123456789abcdef, "\xef\xcd\xab\x89\x67\x45\x23\x01", "\x01\x23\x45\x67\x89\xab\xcd\xef");
    check_integer(s8, -2, "\xfe", "\xfe");
    check_integer(s16, -257, "\xff\xfe", "\xfe\xff");
    check_integer(s32, -0x80000000, "\x00\x00\x00\x80", "\x80\x00\x00\x00");
    check_integer(s64, -0x8000000000000000, "\x00\x00\x00\x00\x00\x00\x00\x80", "\x80\x00\x00\x00\x00\x00\x00\x00");
    bytes := "\xbetail";
    tag := A.asn1_read_tag(*bytes);

    assert(tag.class == .CONTEXT_SPECIFIC && tag.length_value == .CONSTRUCTED && tag.number == 30);
    assert(bytes == "tail");
    bytes = "\x82\x01\x02rest";
    length := A.asn1_read_length(*bytes);

    assert(length == 258 && bytes == "rest");
    bytes = "\x88\xff\xff\xff\xff\xff\xff\xff\xff";
    length = A.asn1_read_length(*bytes);
    assert(length == 0xffffffffffffffff && bytes.count == 0);
    assert(A.asn1_decode_integer("\x7f") == 127);
    assert(A.asn1_decode_integer("\x00\x80") == 128);
    assert(A.asn1_decode_integer("\x00\x00\x7f") == 127);
    assert(A.asn1_decode_integer("\x7f\xff\xff\xff\xff\xff\xff\xff") == 0x7fffffffffffffff);
    oid := A.asn1_decode_object_identifier("\x2a\x86\x48\x86\xf7\x0d\x01\x01\x01");
    text := A.asn1_to_string(oid);

    assert(text == "1.2.840.113549.1.1.1.");
    free(text);
    // First combined value is 2^64+79: arc2 is UINT64_MAX, not a wrapped byte.
    oid = A.asn1_decode_object_identifier("\x82\x80\x80\x80\x80\x80\x80\x80\x80\x4f");
    assert(oid.parts_count == 2 && oid.parts[0] == 2 && oid.parts[1] == 0xffffffffffffffff);
    text = A.asn1_to_string(oid);
    assert(text == "2.18446744073709551615.");
    free(text);
    bytes = "\x34\x12\xfe\xfftail";
    first := A.read(u16, *bytes);
    second := A.read_silly_alias(s16, *bytes);

    assert(first == 0x1234 && second == -2 && bytes == "tail");
    bytes = "\xff\xfe";
    second = A.read(s16, *bytes, BIG_ENDIAN = true);
    assert(second == -2 && bytes.count == 0);
    bytes = "abc remainder";
    part := A.read_block(*bytes, 3);

    assert(part == "abc" && bytes == " remainder");
    bytes = "\x03\x00\x00\x00abc\x00\x00\x00\x00rest";
    original := bytes;

    part = A.read_length_prefixed_block(*bytes);
    assert(part == "abc" && part.data == original.data + 4);
    part = A.read_length_prefixed_block(*bytes);
    assert(part.count == 0 && bytes == "rest");
    print("ok\n");
}
