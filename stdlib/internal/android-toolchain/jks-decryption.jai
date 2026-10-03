jdk_decrypt_private_key :: (encrypted_data: string, password: string) -> bool, string {
    if encrypted_data.count < 40 {
        jks_failure("JKS protected key requires salt and integrity digest");
        return false, "";
    }

    ok, password_bytes := jks_password_bytes(password);
    if !ok return false, "";

    defer jks_release_secret(password_bytes);

    plaintext := alloc_string(encrypted_data.count - 40);
    salt := string.{ 20, encrypted_data.data };
    block := jks_hash(password_bytes, salt);
    offset: s64;

    while offset < plaintext.count {
        amount := plaintext.count - offset;
        if amount > 20 amount = 20;

        for index: 0..amount - 1 {
            plaintext.data[offset + index] = encrypted_data.data[20 + offset + index] ^ block[index];
        }

        offset += amount;
        if offset < plaintext.count block = jks_hash(password_bytes, .{ 20, block.data });
    }

    memset(block.data, 0, 20);
    digest := jks_hash(password_bytes, plaintext);
    expected := string.{ 20, encrypted_data.data + encrypted_data.count - 20 };
    actual := string.{ 20, digest.data };
    valid := jks_equal_digest(actual, expected);

    memset(digest.data, 0, 20);

    if !valid {
        jks_release_secret(plaintext);
        jks_failure("JKS private-key integrity/password mismatch");
        return false, "";
    }

    return true, plaintext;
}

decrypt_private_key :: (encrypted_private_key_info: string, password: string) -> bool, private_key_info: string {
    cursor := encrypted_private_key_info;
    ok, sequence := parse_wire_value(*cursor);

    if !ok return false, "";

    if !jks_der_tag(sequence, 16, .CONSTRUCTED) || cursor.count != 0 {
        jks_failure("JKS encrypted key must be one complete ASN.1 SEQUENCE");
        return false, "";
    }

    cursor = sequence.content;
    valid_algorithm, algorithm := parse_wire_value(*cursor);
    if !valid_algorithm return false, "";

    if !jks_der_tag(algorithm, 16, .CONSTRUCTED) {
        jks_failure("JKS algorithm identifier must be a SEQUENCE");
        return false, "";
    }

    parameters := algorithm.content;
    valid_oid, oid := parse_wire_value(*parameters);

    if !valid_oid return false, "";

    if !jks_der_tag(oid, 6, .PRIMITIVE) {
        jks_failure("JKS algorithm identifier requires an OID");
        return false, "";
    }

    decoded := asn1_decode_object_identifier(oid.content);
    if decoded.parts_count != OID_SUN_JKS_ENCRYPTION.parts_count {
        jks_failure("unsupported JKS key-protection algorithm");
        return false, "";
    }

    for index: 0..decoded.parts_count - 1 {
        if decoded.parts[index] != OID_SUN_JKS_ENCRYPTION.parts[index] {
            jks_failure("unsupported JKS key-protection algorithm");
            return false, "";
        }
    }

    // AlgorithmIdentifier parameters may be absent or ASN.1 NULL.
    if parameters.count > 0 {
        valid_null_value, null_value := parse_wire_value(*parameters);
        if !valid_null_value return false, "";

        if !jks_der_tag(null_value, 5, .PRIMITIVE) || null_value.content.count != 0 || parameters.count != 0 {
            jks_failure("unsupported JKS algorithm parameters");
            return false, "";
        }
    }

    valid_payload, payload := parse_wire_value(*cursor);
    if !valid_payload return false, "";

    if !jks_der_tag(payload, 4, .PRIMITIVE) || cursor.count != 0 {
        jks_failure("JKS encrypted key requires a final OCTET STRING");
        return false, "";
    }

    return jdk_decrypt_private_key(payload.content, password);
}

#scope_file
jks_der_tag :: (value: ASN1_Value, number: u8, form: ASN1_Tag_Length_Value) -> bool {
    return value.class == .UNIVERSAL && value.number == number && value.length_value == form;
}
