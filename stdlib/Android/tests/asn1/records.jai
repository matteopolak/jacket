// Refreshed reviewed ASN1_Value views/consumption, expected tags and u32 prefix.
#import "Basic";
A :: #import "Android/Toolchain";

errors: s64;
assertions: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if message.count > 0 && (info.common_flags & .ERROR) != 0 errors += 1;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    assertions += 1;
    return false;
}

main :: () {
    assert(type_info(A.ASN1_Value).name == "ASN1_Value");
    bytes := "\x30\x03\x02\x01\x7f\x05\x00tail";
    start := bytes;
    // The expected overload ignores primitive/constructed form.
    value := A.asn1_read_value(*bytes, .{ universal_type = .SEQUENCE, class = .UNIVERSAL, length_value = .PRIMITIVE });

    assert(value.universal_type == .SEQUENCE && value.length_value == .CONSTRUCTED);
    assert(value.content == "\x02\x01\x7f" && value.content.data == start.data + 2);
    assert(value.value_data == "\x30\x03\x02\x01\x7f" && value.value_data.data == start.data);
    child_input := value.content;
    child := A.asn1_read_value(*child_input);

    assert(child.universal_type == .INTEGER && child.content == "\x7f" && child_input.count == 0);
    assert(A.asn1_decode_integer(child.content) == 127);
    value = A.asn1_read_value(*bytes);
    assert(value.universal_type == .NULL && value.content.count == 0 && value.value_data == "\x05\x00" && bytes == "tail");

    data: [5] u8 = .[4, 3, 65, 66, 67];
    input: string = .{ data.count, data.data };

    value = A.asn1_read_value(*input);
    data[3] = 90;
    assert(value.content == "AZC" && value.value_data[3] == 90);
    bytes = "\x82\x81\x01xrest";
    value = A.asn1_read_value(*bytes, .{ number = 2, class = .CONTEXT_SPECIFIC });
    assert(value.content == "x" && value.value_data == "\x82\x81\x01x" && bytes == "rest");

    original_logger := context.logger;
    original_assertion := context.assertion_failed;

    for kind: 0..13 {
        input = ifx kind == 0 then "\x02\x01\x01tail" else
            ifx kind == 1 then "\x04\x03x" else ifx kind == 2 then "\x00" else
            ifx kind == 3 then "\x40" else ifx kind == 4 then "\xc0" else
            ifx kind == 5 then "\x08" else ifx kind == 6 then "\xff" else
            ifx kind == 7 then "\x80" else ifx kind == 8 then "" else
            ifx kind == 9 then "\x00\x00\x00\x00\x00\x00\x00\x00\x00" else
            ifx kind == 10 then "\x01\x00" else ifx kind == 11 then "\x03\x00\x00\x00x" else
            ifx kind == 12 then "\x1f" else "\x1d";
        context.logger = capture;
        context.assertion_failed = continue_failure;

        if kind == 0 {
            value = A.asn1_read_value(*input, .{ universal_type = .NULL });
        } else if kind == 1 {
            value = A.asn1_read_value(*input);
        } else if kind >= 7 && kind <= 9 {
            A.asn1_decode_integer(input);
        } else if kind == 10 || kind == 11 {
            A.read_length_prefixed_block(*input);
        } else if kind == 6 {
            A.asn1_read_length(*input);
        } else A.asn1_read_tag(*input);

        context.logger = original_logger;
        context.assertion_failed = original_assertion;
        assert(errors == kind + 1 && assertions == kind + 1);
        if kind == 0 assert(input == "tail" && value.universal_type == .INTEGER);
        if kind == 1 assert(value.value_data.count == 0);
    }

    print("ok\n");
}
