Zip_Writer :: struct {
    File_Info :: struct {
        filename: string;
        offset: u32;
        compression_method: Zip_Compression_Method;
        compressed_size: u32;
        uncompressed_size: u32;
        crc: u32;
    }
    zip_file_infos: [..] File_Info;
    builder: String_Builder;
    dumb_padding_memory: [] u8;
    DUMMY_CRC: u32 : 0x12345678;
    write_crcs := true;
}

zip_init_writer :: (using w: *Zip_Writer, do_crcs := true, max_alignment := 0) {
    if max_alignment < 0 {
        zip_wire_failure("ZIP padding capacity must be nonnegative");
        return;
    }

    write_crcs = do_crcs;
    zip_file_infos.allocator = temporary_allocator;

    if max_alignment > 0 {
        push_allocator(temporary_allocator);
        memory := cast(*u8) alloc(max_alignment);
        if !memory {
            zip_wire_failure("ZIP padding allocation failed");
            return;
        }

        memset(memory, 0, max_alignment);
        dumb_padding_memory = .{ max_alignment, memory };
    }
}

zip_add_file :: (using w: *Zip_Writer, filepath: string, zip_filename: string, compression := Zip_Compression_Method.NONE, alignment := 0) -> bool {
    data, success := Zip_File.read_entire_file(filepath);
    if !success {
        zip_wire_failure("ZIP source file could not be read");
        return false;
    }

    defer free(data);

    return zip_add_file_from_memory(w, data, zip_filename, compression, alignment);
}

zip_add_file_from_memory :: (using w: *Zip_Writer, file_data: string, zip_filename: string, compression := Zip_Compression_Method.NONE, alignment := 0) -> bool {
    if compression != .NONE && compression != .DEFLATE {
        context.logger("ZIP compression method is unsupported", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false;
    }

    if file_data.count < 0 || file_data.count > 0xffffffff || (file_data.count > 0 && !file_data.data) || zip_filename.count < 0 || zip_filename.count > 0xffff || (zip_filename.count > 0 && !zip_filename.data) {
        zip_wire_failure("ZIP32 input size or storage is invalid");
        return false;
    }

    if alignment != 0 && alignment != 4096 {
        zip_wire_failure("ZIP supports only zero or 4096-byte alignment");
        return false;
    }

    offset := builder_string_length(*builder);
    if offset > 0xffffffff || zip_file_infos.count >= 0xffff {
        zip_wire_failure("ZIP32 offsets or entry count exceeded");
        return false;
    }

    padding: s64;

    if alignment != 0 {
        padding = (4096 - (offset + ZIP_LOCAL_HEADER_SIZE + zip_filename.count) % 4096) % 4096;
        if padding > dumb_padding_memory.count || (padding > 0 && !dumb_padding_memory.data) {
            zip_wire_failure("ZIP padding exceeds initialized capacity");
            return false;
        }
    }

    payload := file_data;
    owned_payload: string;

    if compression == .DEFLATE {
        owned_payload = zip_deflate_blocks(file_data);
        payload = owned_payload;
        if !payload.data return false;
    }

    defer if owned_payload.data free(owned_payload);

    if payload.count > 0xffffffff || offset + ZIP_LOCAL_HEADER_SIZE + zip_filename.count + padding + payload.count > 0xffffffff {
        zip_wire_failure("ZIP32 compressed size or output offset exceeded");
        return false;
    }

    crc := DUMMY_CRC;
    if write_crcs crc = zip_crc32(file_data);

    header: Zip_Local_File_Header;
    header.version_needed_to_extract = ZIP_VERSION_DEFAULT;
    header.compression_method = compression;
    header.crc_32 = crc;
    header.compressed_size = cast(u32) payload.count;
    header.uncompressed_size = cast(u32) file_data.count;
    header.filename_length = cast(u16) zip_filename.count;
    header.extra_field_length = cast(u16) padding;
    append_by_pointer(*builder, *header);
    append(*builder, zip_filename);
    append(*builder, dumb_padding_memory.data, padding);
    append(*builder, payload);
    if builder.failed return false;

    array_add(*zip_file_infos, .{ zip_filename, cast(u32) offset, compression,
        cast(u32) payload.count, cast(u32) file_data.count, crc });

    return true;
}

zip_write_central_directory :: (using w: *Zip_Writer) {
    offset := builder_string_length(*builder);
    if offset > 0xffffffff || zip_file_infos.count > 0xffff {
        zip_wire_failure("ZIP32 directory limits exceeded");
        return;
    }

    directory_size: s64;
    for info: zip_file_infos directory_size += size_of(Zip_Central_Directory_Header) + info.filename.count;

    if directory_size > 0xffffffff || offset + directory_size + ZIP_END_OF_CENTRAL_DIRECTORY_RECORD_SIZE > 0xffffffff {
        zip_wire_failure("ZIP32 directory size exceeded");
        return;
    }

    for info: zip_file_infos {
        header: Zip_Central_Directory_Header;
        header.version_made_by = ZIP_VERSION_DEFAULT;
        header.version_needed_to_extract = ZIP_VERSION_DEFAULT;
        header.compression_method = info.compression_method;
        header.crc_32 = info.crc;
        header.compressed_size = info.compressed_size;
        header.uncompressed_size = info.uncompressed_size;
        header.filename_length = cast(u16) info.filename.count;
        header.relative_offset_of_local_header = info.offset;
        append_by_pointer(*builder, *header);
        append(*builder, info.filename);
    }

    end: Zip_End_Of_Central_Directory_Record;
    end.total_entries = cast(u16) zip_file_infos.count;
    end.total_entries_this_disk = end.total_entries;
    end.central_directory_size = cast(u32) directory_size;
    end.central_directory_offset = cast(u32) offset;
    append_by_pointer(*builder, *end);
    if builder.failed zip_wire_failure("ZIP central-directory output allocation failed");
}

#scope_module
Zip_File :: #import "File";

// Raw DEFLATE stored blocks. This is a complete independently written encoder
// with bounded blocks and a real final marker, deliberately trading compression
// ratio for a dependency-free portable representation.
zip_deflate_blocks :: (data: string) -> string {
    blocks := max(cast(s64) 1, (data.count + 65534) / 65535);
    encoded := alloc_string(data.count + blocks * 5);

    if !encoded.data return "";

    input: s64;
    output: s64;

    for block: 0..blocks - 1 {
        length := cast(u16) min(data.count - input, cast(s64) 65535);

        encoded[output] = ifx block + 1 == blocks then 1 else 0;
        encoded[output + 1] = cast(u8) (length & 255);
        encoded[output + 2] = cast(u8) (length >> 8);
        inverse := ~length;

        encoded[output + 3] = cast(u8) (inverse & 255);
        encoded[output + 4] = cast(u8) (inverse >> 8);
        if length > 0 memcpy(encoded.data + output + 5, data.data + input, length);

        output += 5 + cast(s64) length;
        input += length;
    }

    return encoded;
}
