ASN1_Tag_Class :: enum u8 {
    UNIVERSAL :: 0;
    APPLICATION :: 1;
    CONTEXT_SPECIFIC :: 2;
    PRIVATE :: 3;
}

ASN1_Tag_Length_Value :: enum u8 {
    PRIMITIVE :: 0;
    CONSTRUCTED :: 1;
}

ASN1_Universal_Type :: enum u8 {
    RERSERVED_FOR_ENCODING_RULES :: 0;
    SEQUENCE :: 16;
    SET :: 17;
    BOOLEAN :: 1;
    INTEGER :: 2;
    ENUMERATED :: 10;
    REAL :: 9;
    NULL :: 5;
    BIT_STRING :: 3;
    OCTET_STRING :: 4;
    OBJECT_IDENTIFIER :: 6;
    RELATIVE_OBJECT_IDENTIFER :: 13;
    OBJECT_DESCRIPTOR :: 7;
    EMBEDDED_PDV :: 11;
    UTF8_STRING :: 12;
    NUMERIC_STRING :: 18;
    PRINTABLE_STRING :: 19;
    TELETEX_STRING :: 20;
    VIDEOTEX_STRING :: 21;
    IA5_STRING :: 22;
    GRAPHIC_STRING :: 25;
    VISIBLE_STRING :: 26;
    GENERAL_STRING :: 27;
    UNIVERSAL_STRING :: 28;
    BMP_STRING :: 30;
    TIME :: 14;
    UTC_TIME :: 23;
    GENERALIZED_TIME :: 24;
}

ASN1_Tag :: struct {
    union {
        number: u8;
        universal_type: ASN1_Universal_Type;
    }
    length_value: ASN1_Tag_Length_Value;
    class: ASN1_Tag_Class;
}

ASN1_KNOWN_OIDS :: ASN1_Object_Identifier.[OID_SUN_JKS_ENCRYPTION];

asn1_read_tag :: (s: *string) -> ASN1_Tag {
    success, result := parse_wire_tag(s);
    if !success return .{};
    return result;
}

asn1_read_length :: (s: *string) -> u64 {
    success, result := parse_wire_length(s);
    if !success return 0;
    return result;
}

#scope_module
parse_wire_tag :: (s: *string) -> bool, ASN1_Tag {
    result: ASN1_Tag;

    if s.count < 1 {
        wire_failure("missing ASN.1 tag");
        return false, result;
    }

    first := s.data[0];
    number := first & 31;
    tag_class := first >> 6;

    if first == 0 || number == 31 || (tag_class != 0 && tag_class != 2) ||
       (tag_class == 0 && (number == 8 || number == 15 || number == 29)) {
        wire_failure("unsupported ASN.1 identifier");
        return false, result;
    }

    result.number = cast(u8) number;
    result.length_value = cast(ASN1_Tag_Length_Value) ((first >> 5) & 1);
    result.class = cast(ASN1_Tag_Class) (first >> 6);
    s.data += 1;
    s.count -= 1;

    return true, result;
}

parse_wire_length :: (s: *string) -> bool, u64 {
    if s.count < 1 {
        wire_failure("missing ASN.1 length");
        return false, 0;
    }

    first := s.data[0];
    if first < 128 {
        s.data += 1;
        s.count -= 1;
        return true, first;
    }

    count: s64 = first & 127;
    if count == 0 {
        wire_failure("indefinite ASN.1 length is unsupported");
        return false, 0;
    }

    if count > 8 || s.count - 1 < count {
        wire_failure("ASN.1 length is truncated or exceeds u64");
        return false, 0;
    }

    value: u64;
    for index: 1..count value = (value << 8) | s.data[index];

    s.data += count + 1;
    s.count -= count + 1;

    return true, value;
}

wire_failure :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
}
