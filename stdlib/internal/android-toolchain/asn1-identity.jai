ASN1_Object_Identifier :: struct {
    MAX_SUBIDENTIFIERS :: 12;
    parts: [MAX_SUBIDENTIFIERS] u64;
    parts_count := 0;
}

JKS_MAGIC: u32 : 0xfeedfeed;

JKS_MAGIC_LITTLE_ENDIAN :: #run -> u32 {
    return 0xfeedfeed;
}

OID_SUN_JKS_ENCRYPTION :: ASN1_Object_Identifier.{ parts_count = 11, parts = .[1, 3, 6, 1, 4, 1, 42, 2, 17, 1, 1, 0] };

asn1_oid_equals :: (a: ASN1_Object_Identifier, b: ASN1_Object_Identifier) -> bool {
    left := cast(*u8) *a;
    right := cast(*u8) *b;

    for index: 0..size_of(ASN1_Object_Identifier) - 1 {
        if left[index] != right[index] return false;
    }

    return true;
}
