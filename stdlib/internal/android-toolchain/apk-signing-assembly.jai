// Serialization is independent of credentials and crypto-provider ownership.
#scope_module

apk_v2_encode_signed_data :: (digest: string, certificate: string) -> string {
    assert(digest.count == 32, "APK v2 requires a SHA256 digest");
    assert(certificate.count > 0 && certificate.count <= 0xffffffff - 60, "APK certificate length exceeds framing limits");
    result := alloc_string(60 + certificate.count);

    apk_v2_write_u32(result.data, 44);
    apk_v2_write_u32(result.data + 4, 40);
    apk_v2_write_u32(result.data + 8, cast(u32) Algorithm_Id.ECDSA_SHA256);
    apk_v2_write_u32(result.data + 12, 32);
    memcpy(result.data + 16, digest.data, 32);
    apk_v2_write_u32(result.data + 48, cast(u32) (4 + certificate.count));
    apk_v2_write_u32(result.data + 52, cast(u32) certificate.count);
    memcpy(result.data + 56, certificate.data, certificate.count);
    apk_v2_write_u32(result.data + 56 + certificate.count, 0);

    return result;
}

apk_v2_serialize_block :: (digest: string, certificate: string, signature: string, spki: string) -> string {
    assert(signature.count > 0 && signature.count <= 0xffffffff - 16, "APK signature length exceeds framing limits");
    assert(spki.count > 0 && spki.count <= 0xffffffff, "APK public key length exceeds framing limits");
    data := apk_v2_encode_signed_data(digest, certificate);

    defer free(data);
    signer_length := 24 + data.count + signature.count + spki.count;

    assert(signer_length <= 0xffffffff - 4, "APK signer length exceeds framing limits");
    total := 76 + data.count + signature.count + spki.count;
    result := alloc_string(total);

    apk_v2_write_u64(result.data, cast(u64) (total - 8));
    apk_v2_write_u64(result.data + 8, cast(u64) (12 + signer_length));
    apk_v2_write_u32(result.data + 16, SIGNING_BLOCK_ID_V2_SIGNATURE);
    apk_v2_write_u32(result.data + 20, cast(u32) (4 + signer_length));
    apk_v2_write_u32(result.data + 24, cast(u32) signer_length);
    apk_v2_write_u32(result.data + 28, cast(u32) data.count);
    memcpy(result.data + 32, data.data, data.count);
    p := result.data + 32 + data.count;

    apk_v2_write_u32(p, cast(u32) (12 + signature.count));
    apk_v2_write_u32(p + 4, cast(u32) (8 + signature.count));
    apk_v2_write_u32(p + 8, cast(u32) Algorithm_Id.ECDSA_SHA256);
    apk_v2_write_u32(p + 12, cast(u32) signature.count);
    memcpy(p + 16, signature.data, signature.count);
    apk_v2_write_u32(p + 16 + signature.count, cast(u32) spki.count);
    memcpy(p + 20 + signature.count, spki.data, spki.count);
    apk_v2_write_u64(result.data + total - 24, cast(u64) (total - 8));
    memcpy(result.data + total - 16, ANDROID_APK_SIGNING_BLOCK_MAGIC.data, 16);

    return result;
}

apk_v2_assemble :: (apk: Android_Apk, block: string) -> string {
    assert(apk.entire_apk_is_loaded && !apk.is_signed, "APK assembly requires a fully loaded unsigned APK");
    assert(apk.content.count >= 0 && block.count >= 32, "APK assembly requires content and a complete signing block");
    assert(apk.content.count + block.count <= 0xffffffff, "Signed central directory offset exceeds ZIP32");
    assert(apk.central_directory.count >= 0 && apk.central_directory.count <= 0xffffffff, "Central directory exceeds ZIP32");
    assert(apk.end_of_central_directory.count >= 22 && apk.end_of_central_directory.count <= 65557, "APK end record exceeds ZIP comment limits");
    total := apk.content.count + block.count + apk.central_directory.count + apk.end_of_central_directory.count;
    result := alloc_string(total);
    position: s64;

    for section: string.[apk.content, block, apk.central_directory, apk.end_of_central_directory] {
        if section.count > 0 memcpy(result.data + position, section.data, section.count);
        position += section.count;
    }

    end := string.{ apk.end_of_central_directory.count, result.data + total - apk.end_of_central_directory.count };

    zip_eocd_set_start_of_central_directory_offset(end, cast(u32) (apk.content.count + block.count));

    return result;
}
