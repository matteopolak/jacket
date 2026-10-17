apk_read_signing_block_v2 :: (s: *string) -> bool, Apk_Signing_Block_V2 {
    original := s.*;
    remaining := original;
    length_ok := apk_v2_take_length32(*remaining);

    if !length_ok {
        apk_v2_failure("APK v2 signer sequence is truncated");
        return false, .{};
    }

    value := string.{ original.count - remaining.count, original.data };
    valid, result := apk_v2_read_payload(value);

    if !valid {
        apk_v2_failure("Unsupported or malformed APK v2 signer, entries or attributes");
        return false, .{};
    }

    s.* = remaining;

    return true, result;
}

#scope_module

apk_v2_failure :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
}

apk_v2_read_container :: (input: string, verify_structure := false) -> bool, Apk_Signing_Block_V2 {
    if input.count < 32 || !input.data return false, .{};

    size := apk_v2_u64(input.data);
    if size != cast(u64) (input.count - 8) return false, .{};
    if apk_v2_u64(input.data + input.count - 24) != size return false, .{};

    magic := string.{ 16, input.data + input.count - 16 };
    if magic != ANDROID_APK_SIGNING_BLOCK_MAGIC return false, .{};

    pairs := string.{ input.count - 32, input.data + 8 };
    payload: string;
    found := false;

    while pairs.count > 0 {
        length_ok, length_bytes := apk_v2_take(*pairs, 8);
        if !length_ok return false, .{};

        length := apk_v2_u64(length_bytes.data);
        if length < 4 || length > cast(u64) pairs.count return false, .{};

        pair_ok, pair := apk_v2_take(*pairs, cast(s64) length);
        if !pair_ok return false, .{};

        identifier := apk_v2_u32(pair.data);
        if identifier == SIGNING_BLOCK_ID_V2_SIGNATURE {
            if found return false, .{};
            found = true;
            payload = .{ pair.count - 4, pair.data + 4 };
        } else if verify_structure {
            if identifier == SIGNING_BLOCK_ID_V3_SIGNATURE return false, .{};

            if identifier == SIGNING_BLOCK_ID_VERITY_PADDING {
                padding := string.{ pair.count - 4, pair.data + 4 };
                for byte: padding if byte != 0 return false, .{};
            } else {
                // V3.1 follows the unknown-ID path, unlike v3.
                context.logger("Unknown APK signing block ID\n", context.logger_data, .{});
            }
        }
    }

    if !found return false, .{};

    return apk_v2_read_payload(payload);
}

apk_v2_read_payload :: (input: string) -> bool, Apk_Signing_Block_V2 {
    result: Apk_Signing_Block_V2;
    complete := false;

    defer if !complete apk_v2_free_arrays(*result);

    cursor := input;
    signers_ok, signers := apk_v2_take_length32(*cursor);

    if !signers_ok || cursor.count != 0 return false, .{};

    signer_ok, signer := apk_v2_take_length32(*signers);
    if !signer_ok || signers.count != 0 return false, .{};

    data_ok, signed_data := apk_v2_take_length32(*signer);
    if !data_ok return false, .{};

    signatures_ok, signatures := apk_v2_take_length32(*signer);
    if !signatures_ok return false, .{};

    spki_ok, spki := apk_v2_take_length32(*signer);
    if !spki_ok || spki.count == 0 || signer.count != 0 return false, .{};

    cursor = signed_data;
    digests_ok, digests := apk_v2_take_length32(*cursor);
    if !digests_ok return false, .{};

    certificates_ok, certificates := apk_v2_take_length32(*cursor);
    if !certificates_ok return false, .{};

    attributes_ok, attributes := apk_v2_take_length32(*cursor);
    if !attributes_ok || attributes.count != 0 || cursor.count != 0 return false, .{};

    digest_record_ok, digest_record := apk_v2_take_length32(*digests);
    if !digest_record_ok || digests.count != 0 return false, .{};

    digest_id_ok, digest_id := apk_v2_take(*digest_record, 4);
    if !digest_id_ok || apk_v2_u32(digest_id.data) != cast(u32) Algorithm_Id.ECDSA_SHA256 return false, .{};

    digest_ok, digest := apk_v2_take_length32(*digest_record);
    if !digest_ok || digest.count != 32 || digest_record.count != 0 return false, .{};

    signature_record_ok, signature_record := apk_v2_take_length32(*signatures);
    if !signature_record_ok || signatures.count != 0 return false, .{};

    signature_id_ok, signature_id := apk_v2_take(*signature_record, 4);
    if !signature_id_ok || apk_v2_u32(signature_id.data) != cast(u32) Algorithm_Id.ECDSA_SHA256 return false, .{};

    signature_ok, signature := apk_v2_take_length32(*signature_record);
    if !signature_ok || signature.count == 0 || signature_record.count != 0 return false, .{};

    certificate_ok, certificate := apk_v2_take_length32(*certificates);
    if !certificate_ok || certificate.count == 0 || certificates.count != 0 return false, .{};

    array_add(*result.digests, .{ .ECDSA_SHA256, digest });
    array_add(*result.signatures, .{ .ECDSA_SHA256, signature });
    array_add(*result.certificates, .{ certificate });
    result.signed_data = signed_data;
    result.subject_public_key_info = spki;
    complete = true;

    return true, result;
}

apk_v2_free_arrays :: (block: *Apk_Signing_Block_V2) {
    array_free(block.digests);
    array_free(block.signatures);
    array_free(block.certificates);
    block.* = .{};
}

apk_v2_take :: (cursor: *string, length: s64) -> bool, string {
    if cursor.count < 0 || length < 0 || length > cursor.count return false, "";
    if length > 0 && !cursor.data return false, "";

    value := string.{ length, cursor.data };

    if length > 0 cursor.data += length;

    cursor.count -= length;

    return true, value;
}

apk_v2_take_length32 :: (cursor: *string) -> bool, string {
    remaining := cursor.*;
    ok, bytes := apk_v2_take(*remaining, 4);

    if !ok return false, "";

    length := apk_v2_u32(bytes.data);
    value_ok, value := apk_v2_take(*remaining, cast(s64) length);

    if !value_ok return false, "";

    cursor.* = remaining;

    return true, value;
}

apk_v2_u32 :: (bytes: *u8) -> u32 {
    value: u32;
    for index: 0..3 value |= (cast(u32) bytes[index]) << (8 * index);
    return value;
}

apk_v2_u64 :: (bytes: *u8) -> u64 {
    return (cast(u64) apk_v2_u32(bytes)) | ((cast(u64) apk_v2_u32(bytes + 4)) << 32);
}

apk_v2_write_u32 :: (bytes: *u8, value: u32) {
    for index: 0..3 bytes[index] = cast(u8) ((value >> (8 * index)) & 255);
}

apk_v2_write_u64 :: (bytes: *u8, value: u64) {
    for index: 0..7 bytes[index] = cast(u8) ((value >> (8 * index)) & 255);
}
