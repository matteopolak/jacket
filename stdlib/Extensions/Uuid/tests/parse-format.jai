// Parsing and formatting of canonical, uppercase, URN and braced UUIDs, the error kinds and offsets for bad
// input, the nil and max values, and appending to a builder.
#import "Basic";
#import "String";
U :: #import "Extensions/Uuid";

failures := 0;

expect :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

CANONICAL :: "123e4567-e89b-12d3-a456-426614174000";

main :: () {
    using U; // UUID equality is `U.operator ==`.
    // Round trip of the canonical form; version 1 and the RFC variant from the bits of the text.
    id, error := U.parse_uuid(CANONICAL);

    expect(error.kind == .NONE, "canonical text parses");
    expect(U.format_uuid(id) == CANONICAL, "canonical text formats back to itself");
    expect(U.uuid_version(id) == 1, "version nibble of 123e4567-...-426614174000 is 1");
    expect(U.uuid_variant(id) == .RFC, "variant of the RFC layout is .RFC");
    expect(id.bytes[0] == 0x12 && id.bytes[3] == 0x67 && id.bytes[15] == 0x00, "bytes are in network order");

    // Uppercase input parses to the same value and formats in lowercase.
    upper, upper_error := U.parse_uuid("123E4567-E89B-12D3-A456-426614174000");

    expect(upper_error.kind == .NONE && U.uuid_equal(upper, id), "uppercase text parses to the same UUID");
    expect(U.format_uuid(upper) == CANONICAL, "uppercase input formats in lowercase");

    // URN and braced forms.
    urn, urn_error := U.parse_uuid(tprint("urn:uuid:%", CANONICAL));

    expect(urn_error.kind == .NONE && urn == id, "urn:uuid: form parses");
    braced, braced_error := U.parse_uuid(tprint("{%}", CANONICAL));

    expect(braced_error.kind == .NONE && braced == id, "braced form parses");

    // Byte order: the first byte is the first pair of hex digits.
    ordered, ordered_error := U.parse_uuid("00112233-4455-6677-8899-aabbccddeeff");

    expect(ordered_error.kind == .NONE, "all-digits text parses");
    expect(ordered.bytes[0] == 0x00 && ordered.bytes[4] == 0x44 && ordered.bytes[8] == 0x88 && ordered.bytes[15] == 0xff, "byte positions");

    // Errors: the kind, and the byte offset of the problem.
    _, length_error := U.parse_uuid("123e4567-e89b-12d3-a456-42661417400");

    expect(length_error.kind == .LENGTH && length_error.offset == 35, tprint("35 characters: kind % offset %", length_error.kind, length_error.offset));

    _, too_long := U.parse_uuid("123e4567-e89b-12d3-a456-4266141740000");

    expect(too_long.kind == .LENGTH, "37 characters is a length error");

    _, hyphen_error := U.parse_uuid("123e4567xe89b-12d3-a456-426614174000");

    expect(hyphen_error.kind == .SYNTAX && hyphen_error.offset == 8, tprint("missing hyphen: kind % offset %", hyphen_error.kind, hyphen_error.offset));

    _, digit_error := U.parse_uuid("g23e4567-e89b-12d3-a456-426614174000");

    expect(digit_error.kind == .SYNTAX && digit_error.offset == 0, tprint("non-hex digit: kind % offset %", digit_error.kind, digit_error.offset));

    _, brace_error := U.parse_uuid(tprint("{%x", slice(CANONICAL, 0, 36)));

    expect(brace_error.kind == .SYNTAX && brace_error.offset == 37, tprint("unclosed brace: kind % offset %", brace_error.kind, brace_error.offset));

    _, wrong_prefix := U.parse_uuid(tprint("abcdefghi%", CANONICAL));

    expect(wrong_prefix.kind == .LENGTH, "a 45-byte text without the URN prefix is a length error");

    // The nil and max values.
    expect(U.format_uuid(U.UUID_NIL) == "00000000-0000-0000-0000-000000000000", "nil formats as zeros");
    expect(U.format_uuid(U.UUID_MAX) == "ffffffff-ffff-ffff-ffff-ffffffffffff", "max formats as f");
    expect(U.uuid_version(U.UUID_NIL) == 0 && U.uuid_variant(U.UUID_NIL) == .NCS, "nil has version 0 and NCS variant");
    expect(U.uuid_compare(U.UUID_NIL, U.UUID_MAX) == -1 && U.uuid_compare(U.UUID_MAX, U.UUID_NIL) == 1, "compare nil and max");
    expect(U.uuid_compare(id, id) == 0 && id == id, "a UUID equals itself");
    expect(U.uuid_variant(U.UUID_MAX) == .FUTURE, "max has the reserved variant");

    // Appending to a builder gives the same text.
    builder: String_Builder;

    U.append_uuid(*builder, id);
    U.append_uuid(*builder, U.UUID_NIL);
    appended := builder_to_string(*builder);

    defer free(appended);

    expect(appended == tprint("%%", CANONICAL, U.format_uuid(U.UUID_NIL)), "append_uuid appends canonical text");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
