ASN1_Value :: struct {
    using tag: ASN1_Tag;
    content: string;
    value_data: string;
}

asn1_read_value :: (s: *string) -> ASN1_Value {
    success, value := parse_wire_value(s);
    if !success return .{};
    return value;
}

asn1_read_value :: (s: *string, expected: ASN1_Tag) -> ASN1_Value {
    success, value := parse_wire_value(s);
    if !success return .{};

    // Both union members occupy the same byte; form is deliberately unchecked.
    if value.class != expected.class || value.number != expected.number {
        wire_failure("ASN.1 value does not match expected tag class/number");
    }

    return value;
}

#scope_module
parse_wire_value :: (s: *string) -> bool, ASN1_Value {
    start := s.*;
    valid_tag, tag := parse_wire_tag(s);

    if !valid_tag return false, .{};

    valid_length, length := parse_wire_length(s);
    if !valid_length return false, .{};

    if length > cast(u64) s.count {
        wire_failure("ASN.1 value content exceeds input");
        return false, .{};
    }

    value: ASN1_Value;
    value.tag = tag;
    value.content = .{ cast(s64) length, s.data };
    s.data += length;
    s.count -= cast(s64) length;
    value.value_data = .{ start.count - s.count, start.data };

    return true, value;
}
