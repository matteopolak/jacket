x509_get_subject_public_key_info :: (certificate_data: string) -> ASN1_Value {
    encoded := certificate_data;
    valid, certificate := x509_field(*encoded, .SEQUENCE);

    if !valid return .{};

    fields := certificate.content;
    valid_body, body := x509_field(*fields, .SEQUENCE);

    if !valid_body return .{};

    fields = body.content;
    // Version is [0] EXPLICIT with a DEFAULT, unlike the mandatory serial field.
    if fields.count > 0 && fields[0] == 0xa0 {
        valid_version, version := parse_wire_value(*fields);
        if !valid_version return .{};

        version_content := version.content;
        valid_number, number := x509_field(*version_content, .INTEGER);

        if !valid_number return .{};

        if version_content.count != 0 || number.content.count != 1 || number.content[0] > 2 {
            wire_failure("unsupported X.509 certificate version");
            return .{};
        }
    }

    // Skip serial, signature algorithm, issuer, validity and subject structurally.
    // Serials can exceed s64; extracting the key must not decode them as integers.
    for expected: ASN1_Universal_Type.[.INTEGER, .SEQUENCE, .SEQUENCE, .SEQUENCE, .SEQUENCE] {
        valid_field := x509_field(*fields, expected);
        if !valid_field return .{};
    }

    valid_key, key := x509_field(*fields, .SEQUENCE);
    if !valid_key return .{};

    return key;
}

#scope_module
x509_field :: (cursor: *string, expected: ASN1_Universal_Type) -> bool, ASN1_Value {
    valid, value := parse_wire_value(cursor);
    if !valid return false, .{};

    if value.class != .UNIVERSAL || value.universal_type != expected ||
       (expected == .SEQUENCE && value.length_value != .CONSTRUCTED) {
        wire_failure("unexpected X.509 certificate field");
        return false, .{};
    }

    return true, value;
}
