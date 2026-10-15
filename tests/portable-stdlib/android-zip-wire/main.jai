// Reviewed ZIP version/EOCD constants and standard ZIP CRC32 wire behavior.
#import "Basic";
A :: #import "Android/Toolchain";

calls: s64;

once :: () -> int {
    calls += 1;
    return 2;
}

main :: () {
    #assert !#exists(A.zip_wire_u32) && !#exists(A.zip_eocd_valid) && !#exists(A.zip_wire_failure);
    assert(type_info(A.Zip_Version).name == "Zip_Version" && size_of(A.Zip_Version) == 3);
    assert(A.ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIZE == 22 && A.ZIP_EOCD_OFFSET_FIELD_OFFSET == 16);
    assert(type_of(A.ZIP_CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE) == u32);
    assert(type_of(A.ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIGNATURE) == u32);
    assert(A.ZIP_CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE == 0x02014b50);
    assert(A.ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIGNATURE == 0x06054b50);

    for word: 0..65535 {
        version := A.zip_decode_version(cast(u16) word);
        assert(version.mapping == word / 256 && version.major == (word % 256) / 10 && version.minor == (word % 256) % 10);
        assert(A.zip_encode_version(version) == word);
    }

    assert(A.zip_encode_version(.{ mapping = 3, major = 2, minor = 0 }) == 0x0314);
    assert(A.zip_crc32("") == 0 && A.zip_crc32("123456789") == 0xcbf43926);
    bytes: [22] u8;
    bytes[0] = 0x50;
    bytes[1] = 0x4b;
    bytes[2] = 5;
    bytes[3] = 6;
    input := string.{ bytes.count, bytes.data };
    alias := input;

    A.zip_eocd_set_start_of_central_directory_offset(input, 0xfedcba98);
    assert(A.zip_eocd_get_start_of_central_directory_offset(alias) == 0xfedcba98);
    assert(bytes[16] == 0x98 && bytes[17] == 0xba && bytes[18] == 0xdc && bytes[19] == 0xfe);
    for index: 4..15 assert(bytes[index] == 0);

    assert(bytes[20] == 0 && bytes[21] == 0);
    shifted: [26] u8;
    shifted[0] = 0xa5;
    offset: s64 = 1;
    if (cast(u64) shifted.data + cast(u64) offset) % 4 == 0 offset = 2;

    memcpy(shifted.data + offset, bytes.data, bytes.count);
    unaligned := string.{ 22, shifted.data + offset };

    assert(cast(u64) unaligned.data % 4 != 0);
    A.zip_eocd_set_start_of_central_directory_offset(unaligned, 0x01020304);
    assert(A.zip_eocd_get_start_of_central_directory_offset(unaligned) == 0x01020304 && shifted[0] == 0xa5);
    assert(A.Kilobytes(once()) == 2048 && calls == 1);
    assert(A.Megabytes(once()) == 2097152 && calls == 2);
    assert(A.Kilobytes(-2) == -2048 && A.Megabytes(0) == 0);
    print("ok\n");
}
