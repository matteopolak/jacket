Android_Apk :: struct {
    filepath: string;
    zip_dir: Apk_Zip.Zip_Directory;
    entire_apk_is_loaded := true;
    data: string;
    content: string;
    signing_block: string;
    central_directory: string;
    end_of_central_directory: string;
    is_signed := false;
}

Android_Apk_Entry :: struct {
    filename: string;
    compression_method: Zip_Compression_Method;
    compressed_size: u32;
    uncompressed_size: u32;
    offset: u32;
}

ANDROID_APK_SIGNING_BLOCK_MAGIC :: "APK Sig Block 42";

apk_load :: (filepath: string) -> Android_Apk {
    apk: Android_Apk;
    apk.filepath = filepath;
    directory_ok: bool;
    directory_ok, apk.zip_dir = Apk_Zip.load_zip_directory(filepath);
    if !directory_ok {
        zip_wire_failure("APK ZIP directory could not be loaded");
        return .{};
    }

    loaded: bool;
    apk.data, loaded = Zip_File.read_entire_file(filepath);
    if !loaded {
        apk_deinit(*apk);
        zip_wire_failure("APK file could not be read");
        return .{};
    }

    if !apk_partition(*apk) {
        apk_deinit(*apk);
        zip_wire_failure("APK record bounds, signatures or signing-block sizes are invalid");
        return .{};
    }

    return apk;
}

apk_deinit :: (apk: *Android_Apk) {
    Apk_Zip.deinit(*apk.zip_dir);
    free(apk.data);
    memset(apk, 0, size_of(Android_Apk));
} @Releases

for_expansion :: (apk: Android_Apk, body: Code, flags: For_Flags) #expand {
    #assert (flags & .POINTER) == 0, "APK iteration yields values, not pointers";
    assert(apk.entire_apk_is_loaded, "APK iteration requires complete file bytes");

    #if (flags & .REVERSE) != 0 {
        cursor := apk.zip_dir.entries.count;
        while cursor > 0 {
            cursor -= 1;
            `it_index := cursor;
            `it := apk_entry(apk, cursor);
            #insert body;
        }
    } else {
        for index: 0..apk.zip_dir.entries.count - 1 {
            `it_index := index;
            `it := apk_entry(apk, index);
            #insert body;
        }
    }
}

apk_print_sections :: (apk: Android_Apk) {
    print("APK %\nContent: % bytes\nSigning block: % bytes\nCentral directory: % bytes\nEnd record: % bytes\n",
        apk.filepath, apk.content.count, apk.signing_block.count, apk.central_directory.count,
        apk.end_of_central_directory.count);
}

#scope_module
Apk_Zip :: #import "Zip_File_Directory";

apk_partition :: (apk: *Android_Apk) -> bool {
    data := apk.data;
    if data.count < 22 return false;

    end := data.count - 22;
    minimum := max(cast(s64) 0, data.count - 65557);
    found := false;

    while end >= minimum {
        if zip_wire_u32(data.data + end) == ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIGNATURE &&
            end + 22 + apk_u16(data.data + end + 20) == data.count {
            found = true;
            break;
        }

        end -= 1;
    }

    if !found return false;

    directory_offset := cast(s64) zip_wire_u32(data.data + end + 16);
    directory_size := cast(s64) zip_wire_u32(data.data + end + 12);

    if directory_offset > end || directory_size != end - directory_offset || directory_size < 4 return false;
    if zip_wire_u32(data.data + directory_offset) != ZIP_CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE return false;

    apk.central_directory = .{ directory_size, data.data + directory_offset };
    apk.end_of_central_directory = .{ data.count - end, data.data + end };
    content_end := directory_offset;

    if directory_offset >= 24 {
        marker: string = .{ 16, data.data + directory_offset - 16 };
        if marker == ANDROID_APK_SIGNING_BLOCK_MAGIC {
            size := apk_u64(data.data + directory_offset - 24);
            if size < 24 || size > cast(u64) (directory_offset - 8) return false;

            start := directory_offset - cast(s64) size - 8;
            if apk_u64(data.data + start) != size return false;

            apk.is_signed = true;
            apk.signing_block = .{ directory_offset - start, data.data + start };
            content_end = start;
        }
    }

    apk.content = .{ content_end, data.data };

    return true;
}

apk_entry :: (apk: Android_Apk, index: s64) -> Android_Apk_Entry {
    source := apk.zip_dir.entries[index];
    at := cast(s64) source.chunk_offset;

    if at > apk.content.count - ZIP_LOCAL_HEADER_SIZE {
        zip_wire_failure("APK local file header is truncated");
        return .{};
    }

    p := apk.content.data + at;
    if zip_wire_u32(p) != Zip_Local_File_Header.SIGNATURE {
        zip_wire_failure("APK local file header signature is invalid");
        return .{};
    }

    payload := at + ZIP_LOCAL_HEADER_SIZE + apk_u16(p + 26) + apk_u16(p + 28);
    compressed := zip_wire_u32(p + 18);

    if payload > apk.content.count || compressed > cast(u64) (apk.content.count - payload) || payload > 0xffffffff {
        zip_wire_failure("APK local payload is out of bounds");
        return .{};
    }

    return .{ source.filename, cast(Zip_Compression_Method) apk_u16(p + 8), compressed,
        zip_wire_u32(p + 22), cast(u32) payload };
}

apk_u16 :: (p: *u8) -> u16 {
    return (cast(u16) p[0]) | ((cast(u16) p[1]) << 8);
}

apk_u64 :: (p: *u8) -> u64 {
    return (cast(u64) zip_wire_u32(p)) | ((cast(u64) zip_wire_u32(p + 4)) << 32);
}
