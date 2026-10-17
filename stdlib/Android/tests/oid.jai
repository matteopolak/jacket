// Reviewed Android/Toolchain OID shape and complete-record equality.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    assert(type_info(A.ASN1_Object_Identifier).name == "ASN1_Object_Identifier");
    assert(size_of(A.ASN1_Object_Identifier) == 104);
    assert(A.ASN1_Object_Identifier.MAX_SUBIDENTIFIERS == 12);
    assert(type_of(A.JKS_MAGIC) == u32 && type_of(A.JKS_MAGIC_LITTLE_ENDIAN) == u32);
    assert(A.JKS_MAGIC == 0xfeedfeed && A.JKS_MAGIC_LITTLE_ENDIAN == 0xfeedfeed);
    expected := u64.[1, 3, 6, 1, 4, 1, 42, 2, 17, 1, 1, 0];

    assert(A.OID_SUN_JKS_ENCRYPTION.parts_count == 11);
    for value, index: expected assert(A.OID_SUN_JKS_ENCRYPTION.parts[index] == value);

    left := A.OID_SUN_JKS_ENCRYPTION;
    right := left;
    right.parts[11] = 999;
    assert(!A.asn1_oid_equals(left, right));
    assert(A.OID_SUN_JKS_ENCRYPTION.parts[11] == 0);
    right.parts[10] = 999;
    assert(!A.asn1_oid_equals(left, right));

    for count: 0..12 {
        left.parts_count = count;
        right = left;
        assert(A.asn1_oid_equals(left, right));

        if count < 12 {
            right.parts[count] ^= 1;
            assert(!A.asn1_oid_equals(left, right));
            right.parts_count += 1;
            assert(!A.asn1_oid_equals(left, right));
        }

        for index: 0..11 {
            right = left;
            right.parts[index] ^= 1;
            assert(!A.asn1_oid_equals(left, right));
        }
    }

    // Equality treats the count as record bytes, never as an indexing bound.
    for count: s64.[-1, 13, 0x7fffffffffffffff, -0x8000000000000000] {
        left.parts_count = count;
        right = left;
        assert(A.asn1_oid_equals(left, right));

        for index: 0..11 {
            right = left;
            right.parts[index] ^= 1;
            assert(!A.asn1_oid_equals(left, right));
        }

        right = left;
        right.parts_count ^= 1;
        assert(!A.asn1_oid_equals(left, right));
    }

    print("ok\n");
}
