// Encodes entries into an owned byte buffer. Example: bytes, e := encode_archive(items, .ZIP);
encode_archive :: (items: [] Archive_Item, format: Archive_Format, limits := Archive_Limits.{}) -> [] u8, Archive_Error {
    buffer: Archive_Buffer;
    error := write_archive(Archive_Writer.{ data = *buffer, write_proc = buffer_write }, items, format, limits);

    if error.kind != .NONE {
        array_free(buffer.bytes);
        return .[], error;
    }

    return .{ buffer.bytes.count, buffer.bytes.data }, .{};
}

// Writes to a sink without seeking; ZIP uses data descriptors. Deflate buffers one entry.
// Example: error := write_archive(sink, items, .ZIP);
write_archive :: (writer: Archive_Writer, items: [] Archive_Item, format: Archive_Format, limits := Archive_Limits.{}) -> Archive_Error {
    if !valid_limits(limits) || items.count > limits.max_entries return problem(.LIMIT, "invalid limits or entry count");

    total: s64;
    metadata: s64;

    for items {
        if it.data.count < 0 || it.data.count > limits.max_entry || it.data.count > limits.max_total - total return problem(.LIMIT, "entry size limit", 0, it.name);
        if it.name.count > limits.max_metadata - metadata return problem(.LIMIT, "metadata limit", 0, it.name);

        metadata += it.name.count;
        if it.link.count > limits.max_metadata - metadata return problem(.LIMIT, "metadata limit", 0, it.name);

        metadata += it.link.count;
        total += it.data.count;
        if !valid_utf8(cast([] u8) it.name) || !valid_utf8(cast([] u8) it.link) return problem(.INVALID, "invalid UTF-8", 0, it.name);

        for c: it.name {
            if c == 0 return problem(.INVALID, "NUL in name", 0, it.name);
        }

        if it.kind != .FILE && it.data.count return problem(.INVALID, "non-file payload", 0, it.name);
        if it.kind == .OTHER return problem(.UNSUPPORTED, "special archive entry", 0, it.name);
        if format == .ZIP && (it.kind == .SYMLINK || it.kind == .HARDLINK) return problem(.UNSUPPORTED, "writing ZIP links", 0, it.name);
        if it.method != .STORED && it.method != .DEFLATE return problem(.UNSUPPORTED, "compression method", 0, it.name);
    }

    if format == .TAR_GZIP {
        tar, error := encode_archive(items, .TAR, limits);
        if error.kind != .NONE return error;

        defer free(tar.data);
        gzip := Deflate.deflate(tar, .GZIP);

        defer free(gzip.data);
        if gzip.count > limits.max_archive return problem(.LIMIT, "archive output limit");

        if !writer.write_proc(writer.data, gzip) return problem(.IO, "archive write failed");

        return .{};
    }

    w: Archive_Output;
    w.writer = writer;
    w.limit = limits.max_archive;
    if format == .ZIP return zip_write(*w, items);

    return tar_write(*w, items);
}

#scope_module
Archive_Buffer :: struct {
    bytes: [..] u8;
}

Archive_Output :: struct {
    writer: Archive_Writer;
    at: s64;
    limit: s64;
    error: Archive_Error;
}

buffer_write :: (data: *void, bytes: [] u8) -> bool {
    b := cast(*Archive_Buffer) data;

    array_reserve(*b.bytes, b.bytes.count + bytes.count);
    if bytes.count memcpy(b.bytes.data + b.bytes.count, bytes.data, bytes.count);

    b.bytes.count += bytes.count;

    return true;
}

emit :: (w: *Archive_Output, b: [] u8) {
    if w.error.kind != .NONE return;

    if b.count > w.limit - w.at {
        w.error = problem(.LIMIT, "archive output limit", w.at);
        return;
    }

    if !w.writer.write_proc(w.writer.data, b) {
        w.error = problem(.IO, "archive write failed", w.at);
        return;
    }

    w.at += b.count;
}

put :: (b: [] u8, p: s64, value: u64, width: s64) {
    for 0..width - 1 b[p + it] = cast,trunc(u8) (value >> cast(u64) (8 * it));
}

zip_write :: (w: *Archive_Output, items: [] Archive_Item) -> Archive_Error {
    central: Archive_Buffer;

    defer array_free(central.bytes);
    c := Archive_Output.{ writer = .{ data = *central, write_proc = buffer_write }, limit = w.limit };

    for items {
        if it.name.count > 65535 return problem(.LIMIT, "ZIP name too long", w.at, it.name);

        packed: [] u8 = it.data;
        if it.method == .DEFLATE packed = Deflate.deflate(it.data, .RAW);

        defer {
            if it.method == .DEFLATE free(packed.data);
        }

        size := cast(u64) it.data.count;
        compressed := cast(u64) packed.count;
        offset := cast(u64) w.at;
        wide := size >= 0xffffffff || compressed >= 0xffffffff;
        wide_offset := offset >= 0xffffffff;
        crc := Deflate.crc32(it.data);
        h: [30] u8;

        put(h, 0, 0x04034b50, 4);
        put(h, 4, ifx wide then 45 else 20, 2);
        put(h, 6, 0x808, 2);
        put(h, 8, cast(u64) it.method, 2);
        put(h, 12, 33, 2);
        put(h, 26, cast(u64) it.name.count, 2);

        if wide {
            put(h, 18, 0xffffffff, 4);
            put(h, 22, 0xffffffff, 4);
            put(h, 28, 20, 2);
        }

        emit(w, h);
        emit(w, cast([] u8) it.name);

        if wide {
            x: [20] u8;

            put(x, 0, 1, 2);
            put(x, 2, 16, 2);
            put(x, 4, size, 8);
            put(x, 12, compressed, 8);
            emit(w, x);
        }

        emit(w, packed);
        descriptor: [24] u8;

        put(descriptor, 0, 0x08074b50, 4);
        put(descriptor, 4, cast(u64) crc, 4);
        put(descriptor, 8, compressed, ifx wide then 8 else 4);
        put(descriptor, ifx wide then 16 else 12, size, ifx wide then 8 else 4);
        emit(w, slice(descriptor, 0, ifx wide then 24 else 16));
        ch: [46] u8;

        put(ch, 0, 0x02014b50, 4);
        put(ch, 4, 0x32d, 2);
        put(ch, 6, ifx wide || wide_offset then 45 else 20, 2);
        put(ch, 8, 0x808, 2);
        put(ch, 10, cast(u64) it.method, 2);
        put(ch, 14, 33, 2);
        put(ch, 16, cast(u64) crc, 4);
        put(ch, 20, ifx wide then 0xffffffff else compressed, 4);
        put(ch, 24, ifx wide then 0xffffffff else size, 4);
        put(ch, 28, cast(u64) it.name.count, 2);
        extra_length: s64;
        if wide || wide_offset extra_length = 4 + (ifx wide then 16 else 0) + (ifx wide_offset then 8 else 0);

        put(ch, 30, cast(u64) extra_length, 2);
        put(ch, 38, ifx it.kind == .DIRECTORY then 0x41ed0010 else 0x81a40000, 4);
        put(ch, 42, ifx wide_offset then 0xffffffff else offset, 4);
        emit(*c, ch);
        emit(*c, cast([] u8) it.name);

        if extra_length {
            x: [28] u8;

            put(x, 0, 1, 2);
            put(x, 2, cast(u64) extra_length - 4, 2);
            p: s64 = 4;

            if wide {
                put(x, p, size, 8);
                put(x, p + 8, compressed, 8);
                p += 16;
            }

            if wide_offset put(x, p, offset, 8);

            emit(*c, slice(x, 0, extra_length));
        }

        if w.error.kind != .NONE {
            w.error.entry = copy_string(it.name);
            return w.error;
        }

        if c.error.kind != .NONE {
            c.error.entry = copy_string(it.name);
            return c.error;
        }
    }

    start := cast(u64) w.at;

    emit(w, central.bytes);
    count := cast(u64) items.count;
    size := cast(u64) central.bytes.count;
    wide := start >= 0xffffffff || size >= 0xffffffff || count >= 65535;

    if wide {
        end := cast(u64) w.at;
        z: [56] u8;

        put(z, 0, 0x06064b50, 4);
        put(z, 4, 44, 8);
        put(z, 12, 45, 2);
        put(z, 14, 45, 2);
        put(z, 24, count, 8);
        put(z, 32, count, 8);
        put(z, 40, size, 8);
        put(z, 48, start, 8);
        emit(w, z);
        locator: [20] u8;

        put(locator, 0, 0x07064b50, 4);
        put(locator, 8, end, 8);
        put(locator, 16, 1, 4);
        emit(w, locator);
    }

    end: [22] u8;

    put(end, 0, 0x06054b50, 4);
    put(end, 8, min(count, 65535), 2);
    put(end, 10, min(count, 65535), 2);
    put(end, 12, min(size, 0xffffffff), 4);
    put(end, 16, min(start, 0xffffffff), 4);
    emit(w, end);

    return w.error;
}

octal :: (h: [] u8, at: s64, width: s64, value: u64) {
    n := value;

    for < i: 0..width - 2 {
        h[at + i] = cast(u8) (48 + (n & 7));
        n >>= 3;
    }
}

tar_header :: (w: *Archive_Output, name: string, link: string, size: s64, mtime: s64, kind: u8) {
    h: [512] u8;
    if name.count memcpy(h.data, name.data, min(name.count, 100));

    octal(h, 100, 8, 420);
    octal(h, 108, 8, 0);
    octal(h, 116, 8, 0);
    octal(h, 124, 12, cast(u64) size);
    octal(h, 136, 12, cast(u64) max(mtime, 0));
    for 148..155 h[it] = 32;

    h[156] = kind;
    if link.count memcpy(h.data + 157, link.data, min(link.count, 100));

    memcpy(h.data + 257, "ustar\0".data, 6);
    memcpy(h.data + 263, "00".data, 2);
    sum: u64;
    for h sum += cast(u64) it;

    octal(h, 148, 7, sum);
    h[155] = 32;
    emit(w, h);
}

pax_record :: (builder: *String_Builder, key: string, value: string) {
    length := key.count + value.count + 3;

    while true {
        digits := tprint("%", length).count;
        next := key.count + value.count + 3 + digits;
        if next == length break;
        length = next;
    }

    print(builder, "% %=%\n", length, key, value);
}

tar_write :: (w: *Archive_Output, items: [] Archive_Item) -> Archive_Error {
    zero: [1024] u8;

    for items {
        pax: String_Builder;

        defer free_buffers(*pax);
        if it.name.count > 100 pax_record(*pax, "path", it.name);
        if it.link.count > 100 return problem(.UNSUPPORTED, "writing long TAR linkpath", w.at, it.name);
        if it.data.count > 0x1ffffffff pax_record(*pax, "size", tprint("%", it.data.count));
        if it.mtime < 0 || it.mtime > 0x1ffffffff return problem(.UNSUPPORTED, "writing out-of-range TAR mtime", w.at, it.name);

        if builder_string_length(*pax) {
            text := builder_to_string(*pax);

            defer free(text);
            tar_header(w, "PaxHeader", "", text.count, 0, #char "x");
            emit(w, cast([] u8) text);
            emit(w, slice(zero, 0, (512 - text.count % 512) % 512));
        }

        kind := ifx it.kind == .DIRECTORY then #char "5" else ifx it.kind == .SYMLINK then #char "2" else ifx it.kind == .HARDLINK then #char "1" else #char "0";

        tar_header(w, it.name, it.link, ifx it.data.count > 0x1ffffffff then 0 else it.data.count, it.mtime, cast(u8) kind);
        emit(w, it.data);
        emit(w, slice(zero, 0, (512 - it.data.count % 512) % 512));

        if w.error.kind != .NONE {
            w.error.entry = copy_string(it.name);
            return w.error;
        }
    }

    emit(w, zero);

    return w.error;
}
