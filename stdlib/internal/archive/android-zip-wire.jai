// Is a ZIP version word split into the host mapping, and major and minor numbers.
Zip_Version :: struct {
    mapping: u8;
    major: u8;
    minor: u8;
}

// Splits `version_word` into the host byte and the major and minor version numbers (the low byte as tens and
// units).
zip_decode_version :: (version_word: u16) -> Zip_Version {
    low := version_word & 255;
    return .{ cast(u8) (version_word >> 8), cast(u8) (low / 10), cast(u8) (low % 10) };
}

// Packs `version` into a ZIP version word; fails an assertion and returns 0 when the numbers do not fit.
zip_encode_version :: (version: Zip_Version) -> u16 {
    low := (cast(u32) version.major) * 10 + version.minor;
    if version.minor > 9 || low > 255 {
        zip_wire_failure("ZIP version digits exceed their byte representation");
        return 0;
    }

    return ((cast(u16) version.mapping) << 8) | cast(u16) low;
}

ZIP_CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE: u32 : 0x02014b50;
ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIGNATURE: u32 : 0x06054b50;
ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIZE :: 22;
ZIP_EOCD_OFFSET_FIELD_OFFSET :: 16;

// Returns the offset of the central directory stored in the end record `eocd`.
//
// Returns 0 when `eocd` is not a complete end record; that also logs an error and fails an assertion.
zip_eocd_get_start_of_central_directory_offset :: (eocd: string) -> u32 {
    if !zip_eocd_valid(eocd) return 0;
    return zip_wire_u32(eocd.data + ZIP_EOCD_OFFSET_FIELD_OFFSET);
}

// Overwrites the central directory offset inside the end record `eocd`, in place, with `new_offset`.
//
// Changes nothing when `eocd` is not a complete end record; that also logs an error and fails an assertion.
zip_eocd_set_start_of_central_directory_offset :: (eocd: string, new_offset: u32) {
    if !zip_eocd_valid(eocd) return;
    for index: 0..3 eocd.data[ZIP_EOCD_OFFSET_FIELD_OFFSET + index] = cast(u8) ((new_offset >> (index * 8)) & 255);
}

// Returns the CRC-32 of `s`, as ZIP stores it.
zip_crc32 :: (s: string) -> u32 {
    if s.count < 0 || (s.count > 0 && !s.data) {
        zip_wire_failure("ZIP CRC requires readable string storage");
        return 0;
    }

    return Deflate.crc32(cast([] u8) s);
}

// Returns `x` kibibytes in bytes.
Kilobytes :: (x: int) -> int #expand {
    return x * 1024;
}

// Returns `x` mebibytes in bytes.
Megabytes :: (x: int) -> int #expand {
    return x * 1048576;
}

#scope_module
#import "Basic";
Deflate :: #import "Extensions/Deflate";

zip_eocd_valid :: (eocd: string) -> bool {
    if eocd.count < ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIZE || !eocd.data {
        zip_wire_failure("ZIP end record is truncated");
        return false;
    }

    if zip_wire_u32(eocd.data) != ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIGNATURE {
        zip_wire_failure("ZIP end record has invalid signature");
        return false;
    }

    return true;
}

zip_wire_u32 :: (data: *u8) -> u32 {
    result: u32;
    for index: 0..3 result |= (cast(u32) data[index]) << (index * 8);
    return result;
}

zip_wire_failure :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
}
