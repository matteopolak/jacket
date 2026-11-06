// Application metadata uses lowercase protocol names; grpc-* names are reserved.
// Example: `assert(valid_metadata_name("trace-bin"));`.
valid_metadata_name :: (name: string) -> bool {
    if name.count == 0 || begins_with(name, "grpc-") return false;
    for reserved: string.["content-type", "content-length", "content-encoding", "te", "user-agent", "host", "connection", "keep-alive", "proxy-connection", "transfer-encoding", "upgrade"] if name == reserved return false;

    for name {
        if (it >= #char "a" && it <= #char "z") || (it >= #char "0" && it <= #char "9") || it == #char "_" || it == #char "-" || it == #char "." continue;
        return false;
    }

    return true;
}

// Encodes application metadata into temporary header storage. Binary values use
// unpadded Base64; repeated keys retain their order. Fields need no explicit free.
// Example: `fields, valid := encode_metadata(Metadata_Entry.[.{ "x-key", "value" }]);`.
encode_metadata :: (metadata: [] Metadata_Entry, limit := 64 * 1024) -> fields: [] Http.Header_Field, ok: bool {
    fields, failure := metadata_encode(metadata, limit);
    return fields, failure == .NONE;
}

// Copies application metadata into owned storage. Comma-joined binary values
// are split before padded/unpadded Base64 decode. Reserved protocol fields are
// excluded. Release the result with free_metadata, including on an empty result.
// Example: `entries, valid := decode_metadata(fields); defer free_metadata(*entries);`.
decode_metadata :: (fields: [] Http.Header_Field, limit := 64 * 1024) -> metadata: [..] Metadata_Entry, ok: bool {
    metadata, failure := metadata_decode(fields, limit);
    return metadata, failure == .NONE;
}

// Releases every entry's owned name/value, then the array; safe repeatedly.
// Example: `free_metadata(*entries);`.
free_metadata :: (metadata: *[..] Metadata_Entry) {
    for metadata.* {
        if it.name.data free(it.name.data);
        if it.value.data free(it.value.data);
    }

    array_reset(metadata);
}

#scope_module
metadata_base64_valid :: (value: string) -> bool {
    padding := 0;
    symbols := 0;

    for value {
        if it == #char "=" {
            padding += 1;
            if padding > 2 return false;
            continue;
        }

        if padding != 0 return false;
        if !((it >= #char "A" && it <= #char "Z") || (it >= #char "a" && it <= #char "z") || (it >= #char "0" && it <= #char "9") || it == #char "+" || it == #char "/") return false;

        symbols += 1;
    }

    if symbols % 4 == 1 return false;
    if padding > 0 && (symbols + padding) % 4 != 0 return false;

    return true;
}

metadata_headers_fit :: (fields: [] Http.Header_Field, limit: s64) -> bool {
    remaining := limit;

    for fields {
        if remaining < 32 return false;

        remaining -= 32;
        if it.name.count > remaining return false;

        remaining -= it.name.count;
        if it.value.count > remaining return false;

        remaining -= it.value.count;
    }

    return remaining >= 0;
}

Metadata_Error :: enum u8 {
    NONE;
    INVALID;
    LIMIT;
}

metadata_encode :: (metadata: [] Metadata_Entry, limit := 64 * 1024) -> fields: [] Http.Header_Field, failure: Metadata_Error {
    if limit < 0 return .[], .LIMIT;

    out: [..] Http.Header_Field;
    out.allocator = temp;
    total: s64;

    for metadata {
        if !valid_metadata_name(it.name) return .[], .INVALID;

        value := it.value;
        if it.name.count > limit || value.count > limit return .[], .LIMIT;

        binary := ends_with(it.name, "-bin");
        encoded_size := ifx binary then (value.count / 3) * 4 + (ifx value.count % 3 == 0 then 0 else value.count % 3 + 1) else value.count;

        total += it.name.count + encoded_size + 32;
        if total > limit return .[], .LIMIT;
        if binary value = Base64.base64_encode(value, pad = false,, temp);
        else for byte: value if byte < 32 || byte > 126 return .[], .INVALID;

        array_add(*out, .{ name = it.name, value = value });
    }

    return out, .NONE;
}

metadata_decode :: (fields: [] Http.Header_Field, limit := 64 * 1024) -> metadata: [..] Metadata_Entry, failure: Metadata_Error {
    if !metadata_headers_fit(fields, limit) return .{}, .LIMIT;

    out: [..] Metadata_Entry;

    for fields {
        if !valid_metadata_name(it.name) continue;

        if ends_with(it.name, "-bin") {
            for item: split(it.value, ",",, temp) {
                start := 0;
                stop := item.count;

                while start < stop && (item[start] == 32 || item[start] == 9) start += 1;
                while stop > start && (item[stop - 1] == 32 || item[stop - 1] == 9) stop -= 1;

                encoded := slice(item, start, stop - start);
                if !metadata_base64_valid(encoded) {
                    free_metadata(*out);
                    return .{}, .INVALID;
                }

                value, valid := Base64.base64_decode(encoded);
                if !valid {
                    if value.data free(value.data);
                    free_metadata(*out);
                    return .{}, .INVALID;
                }

                array_add(*out, .{ copy_string(it.name), value });
            }
        } else array_add(*out, .{ copy_string(it.name), copy_string(it.value) });
    }

    return out, .NONE;
}

protobuf_content_type :: (value: string) -> bool {
    _, base, _ := split_from_left(value, #char ";");
    base = trim(base);
    return equal_nocase(base, "application/grpc") || equal_nocase(base, "application/grpc+proto");
}
