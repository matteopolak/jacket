// Is one parsed DER value: its tag, the content bytes, and the whole encoding including tag and length.
//
// The strings point into the parsed data.
ASN1_Value :: struct {
    using tag: ASN1_Tag;
    content: string;
    value_data: string;
}

// Reads one DER value from the front of `s` and advances `s` past it.
//
// Returns an empty value when the tag or length is malformed or the content runs past the end of `s`; that
// also logs an error and fails an assertion.
asn1_read_value :: (s: *string) -> ASN1_Value {
    success, value := parse_wire_value(s);
    if !success return .{};
    return value;
}

// Reads one DER value as [asn1_read_value] does, and requires its tag class and number to match `expected`.
//
// The constructed or primitive form is not checked. A mismatch logs an error and fails an assertion; the value
// is still returned.
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
