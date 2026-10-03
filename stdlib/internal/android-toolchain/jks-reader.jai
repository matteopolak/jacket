jks_read :: (data: string, password: string) -> bool, certificate: string, encrypted_private_key_info: string {
    if data.count < 32 {
        jks_failure("JKS input is shorter than header and integrity digest");
        return false, "", "";
    }

    valid_password, password_bytes := jks_password_bytes(password);
    if !valid_password return false, "", "";

    defer jks_release_secret(password_bytes);

    body := string.{ data.count - 20, data.data };
    cursor := body;
    ok, magic := jks_word(*cursor, 4);

    if !ok return false, "", "";

    valid_version, version := jks_word(*cursor, 4);
    if !valid_version return false, "", "";

    valid_entries, entries := jks_word(*cursor, 4);
    if !valid_entries return false, "", "";

    if magic != JKS_MAGIC || (version != 1 && version != 2) || entries != 1 {
        jks_failure("JKS requires supported magic/version and one private-key entry");
        return false, "", "";
    }

    valid_kind, kind := jks_word(*cursor, 4);
    if !valid_kind return false, "", "";

    if kind != 1 {
        jks_failure("JKS entry is not a private key");
        return false, "", "";
    }

    valid_alias := jks_prefixed(*cursor, 2);
    if !valid_alias return false, "", "";

    valid_timestamp := jks_take(*cursor, 8);
    if !valid_timestamp return false, "", "";

    valid_encrypted, encrypted := jks_prefixed(*cursor, 4);
    if !valid_encrypted return false, "", "";

    valid_chain_count, chain_count := jks_word(*cursor, 4);
    if !valid_chain_count return false, "", "";

    if chain_count > 0x7fffffff || (chain_count > 0 && version != 2) {
        jks_failure("JKS certificate chains require version 2 and a nonnegative count");
        return false, "", "";
    }

    last_certificate: string;

    for 0..cast(s64) chain_count - 1 {
        valid_encoding, encoding := jks_prefixed(*cursor, 2);
        if !valid_encoding return false, "", "";

        if encoding != "X.509" {
            jks_failure("JKS certificate encoding must be X.509");
            return false, "", "";
        }

        ok, last_certificate = jks_prefixed(*cursor, 4);
        if !ok return false, "", "";
    }

    if cursor.count != 0 {
        jks_failure("JKS trailing data before integrity digest");
        return false, "", "";
    }

    prefix := jks_join(password_bytes, "Mighty Aphrodite");

    defer jks_release_secret(prefix);
    digest := jks_hash(prefix, body);
    expected := string.{ 20, data.data + body.count };
    actual := string.{ 20, digest.data };

    if !jks_equal_digest(actual, expected) {
        jks_failure("JKS store integrity/password mismatch");
        return false, "", "";
    }

    return true, last_certificate, encrypted;
}

#scope_module
jks_failure :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
}

jks_take :: (cursor: *string, count: s64) -> bool, string {
    if count < 0 || cursor.count < count {
        jks_failure("truncated or negative JKS field");
        return false, "";
    }

    result := string.{ count, cursor.data };

    cursor.data += count;
    cursor.count -= count;

    return true, result;
}

jks_word :: (cursor: *string, width: s64) -> bool, u64 {
    ok, bytes := jks_take(cursor, width);
    if !ok return false, 0;

    value: u64;
    for byte: bytes value = (value << 8) | byte;

    return true, value;
}

jks_prefixed :: (cursor: *string, width: s64) -> bool, string {
    ok, count := jks_word(cursor, width);
    if !ok return false, "";

    if count > 0x7fffffff {
        jks_failure("negative JKS length");
        return false, "";
    }

    return jks_take(cursor, cast(s64) count);
}

jks_password_bytes :: (password: string) -> bool, string {
    if password.count < 0 || password.count > 0x3fffffffffffffff {
        jks_failure("JKS password length overflow");
        return false, "";
    }

    for byte: password {
        if byte > 127 {
            jks_failure("JKS password conversion supports ASCII only");
            return false, "";
        }
    }

    result := alloc_string(password.count * 2);

    for index: 0..password.count - 1 {
        result.data[2 * index] = 0;
        result.data[2 * index + 1] = password[index];
    }

    return true, result;
}

jks_join :: (prefix: string, payload: string) -> string {
    if prefix.count < 0 || payload.count < 0 || prefix.count > 0x7fffffffffffffff - payload.count {
        jks_failure("JKS hash input length overflow");
        return "";
    }

    result := alloc_string(prefix.count + payload.count);
    if prefix.count > 0 memcpy(result.data, prefix.data, prefix.count);
    if payload.count > 0 memcpy(result.data + prefix.count, payload.data, payload.count);

    return result;
}

jks_hash :: (prefix: string, payload: string) -> [20] u8 {
    input := jks_join(prefix, payload);
    defer jks_release_secret(input);
    return sha1(input);
}

jks_release_secret :: (secret: string) {
    if secret.count > 0 memset(secret.data, 0, secret.count);
    free(secret);
}

jks_equal_digest :: (a: string, b: string) -> bool {
    if a.count != b.count return false;
    difference: u8;
    for index: 0..a.count - 1 difference |= a[index] ^ b[index];
    return difference == 0;
}
