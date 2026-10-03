apk_compute_digest_libressl :: (apk: Android_Apk, algorithm: Digest_Hash_Algorithm) -> [SHA256_DIGEST_LENGTH] u8 {
    assert(algorithm == .SHA256, "APK digest supports SHA256 only");
    assert(apk.entire_apk_is_loaded, "APK digest requires the entire APK");
    assert(apk.content.count >= 0 && apk.content.count <= 0xffffffff, "APK content exceeds ZIP32 offset range");
    assert(apk.central_directory.count >= 0, "APK central directory has invalid length");
    assert(apk.end_of_central_directory.count >= ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIZE, "APK end record is truncated");
    if algorithm != .SHA256 || !apk.entire_apk_is_loaded return .{};

    original := zip_eocd_get_start_of_central_directory_offset(apk.end_of_central_directory);

    zip_eocd_set_start_of_central_directory_offset(apk.end_of_central_directory, cast(u32) apk.content.count);
    defer zip_eocd_set_start_of_central_directory_offset(apk.end_of_central_directory, original);

    sections := string.[apk.content, apk.central_directory, apk.end_of_central_directory];
    chunk_count: u64;

    for section: sections {
        assert(section.count >= 0 && (section.count == 0 || section.data != null), "APK digest requires readable section storage");
        chunk_count += cast(u64) (section.count / APK_V2_CHUNK_SIZE);
        if section.count % APK_V2_CHUNK_SIZE != 0 chunk_count += 1;
    }

    assert(chunk_count <= 0xffffffff, "APK chunk count exceeds uint32");

    root := alloc_string(5 + cast(s64) chunk_count * SHA256_DIGEST_LENGTH);

    defer free(root);
    root.data[0] = 0x5a;
    apk_v2_write_u32(root.data + 1, cast(u32) chunk_count);
    // One reusable heap buffer avoids a megabyte on the stack for each chunk.
    leaf := alloc_string(5 + APK_V2_CHUNK_SIZE);

    defer free(leaf);
    leaf.data[0] = 0xa5;
    output_offset := 5;

    for section: sections {
        position: s64;
        while position < section.count {
            count := min(APK_V2_CHUNK_SIZE, section.count - position);

            apk_v2_write_u32(leaf.data + 1, cast(u32) count);
            memcpy(leaf.data + 5, section.data + position, count);
            hash := sha256(string.{ count + 5, leaf.data });

            memcpy(root.data + output_offset, hash.data, SHA256_DIGEST_LENGTH);
            output_offset += SHA256_DIGEST_LENGTH;
            position += count;
        }
    }

    return sha256(root);
}

#scope_module
APK_V2_CHUNK_SIZE :: 1048576;
SHA256_DIGEST_LENGTH :: 32;
