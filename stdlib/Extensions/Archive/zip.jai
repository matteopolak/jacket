#scope_module

zip_index :: (a: *Archive) -> Archive_Error {
    tail_size := min(a.source.size, 65557);
    tail, error := region(a, a.source.size - tail_size, tail_size);

    if error.kind != .NONE return error;

    defer free_archive(*tail);
    b := tail.bytes;
    end := b.count - 22;

    while end >= 0 {
        if u32le(b, end) == 0x06054b50 {
            if u16le(b, end + 20) > b.count - end - 22 return problem(.INVALID, "truncated ZIP comment", a.source.size - tail_size + end);
            break;
        }

        end -= 1;
    }

    if end < 0 return problem(.INVALID, "missing ZIP end record");

    absolute_end := a.source.size - tail_size + end;
    if u16le(b, end + 4) || u16le(b, end + 6) || u16le(b, end + 8) != u16le(b, end + 10) return problem(.UNSUPPORTED, "multi-disk ZIP", absolute_end);

    count := cast(u64) u16le(b, end + 10);
    size := cast(u64) u32le(b, end + 12);
    start := cast(u64) u32le(b, end + 16);
    directory_end := absolute_end;

    if count == 65535 || size == 0xffffffff || start == 0xffffffff {
        locator: [20] u8;

        error = fetch_at(a, absolute_end - 20, locator);
        if error.kind != .NONE return error;
        if u32le(locator, 0) != 0x07064b50 || u32le(locator, 4) || u32le(locator, 16) != 1 return problem(.INVALID, "invalid ZIP64 locator", absolute_end - 20);

        pos := u64le(locator, 8);
        if pos > cast(u64) a.source.size return problem(.INVALID, "invalid ZIP64 offset", absolute_end - 20);

        header: [56] u8;

        error = fetch_at(a, cast(s64) pos, header);
        if error.kind != .NONE return error;

        length := u64le(header, 4);
        if u32le(header, 0) != 0x06064b50 || length < 44 || length > cast(u64) a.limits.max_metadata || length > cast(u64) max(absolute_end - 32, 0) || pos != cast(u64) max(absolute_end - 32, 0) - length return problem(.INVALID, "invalid ZIP64 end record", cast(s64) pos);
        if u32le(header, 16) || u32le(header, 20) || u64le(header, 24) != u64le(header, 32) return problem(.UNSUPPORTED, "multi-disk ZIP64", cast(s64) pos);

        count = u64le(header, 32);
        size = u64le(header, 40);
        start = u64le(header, 48);
        directory_end = cast(s64) pos;
    }

    if count > cast(u64) a.limits.max_entries || size > cast(u64) a.limits.max_metadata return problem(.LIMIT, "ZIP directory limit", absolute_end);
    if size > cast(u64) directory_end || start > cast(u64) directory_end - size return problem(.INVALID, "invalid central directory range", absolute_end);

    // Offsets in a self-extracting ZIP may be relative to the appended ZIP rather than the file.
    base := directory_end - cast(s64) size - cast(s64) start;

    start += cast(u64) base;
    directory, failure := region(a, cast(s64) start, cast(s64) size);
    if failure.kind != .NONE return failure;

    defer free_archive(*directory);
    d := directory.bytes;
    at: s64;

    for 0..cast(s64) count - 1 {
        e, consumed, entry_error := zip_central_entry(a, d, at, start, base);
        if entry_error.kind != .NONE return entry_error;

        error = admit(a, e);
        if error.kind != .NONE {
            free(e.name);
            return error;
        }

        at += consumed;
    }

    if at != d.count return problem(.INVALID, "central directory size mismatch", cast(s64) start + at);

    spans: [..] Archive_Entry;

    defer array_free(spans);
    for a.entries array_add(*spans, it);

    quick_sort(spans, (x: Archive_Entry, y: Archive_Entry) -> int {
        return ifx x.record_start < y.record_start then -1 else ifx x.record_start > y.record_start then 1 else 0;
    });
    finish: s64;

    for spans {
        if it.record_start < finish return problem(.INVALID, "overlapping ZIP entries", it.record_start, it.name);
        finish = it.record_end;
    }

    return .{};
}

zip_local :: (a: *Archive, e: *Archive_Entry, flags: u16, name: [] u8, central: s64, wide: bool) -> Archive_Error {
    wide := wide;
    h: [30] u8;
    error := fetch_at(a, e.record_start, h);

    if error.kind != .NONE return error;
    if u32le(h, 0) != 0x04034b50 || u16le(h, 6) != flags || u16le(h, 8) != cast(u16) e.method return problem(.INVALID, "local/central header mismatch", e.record_start, e.name);

    wide = wide || u32le(h, 18) == 0xffffffff || u32le(h, 22) == 0xffffffff;
    n := cast(s64) u16le(h, 26);
    x := cast(s64) u16le(h, 28);

    if !fits(central, e.record_start, 30 + n + x) return problem(.INVALID, "local header overlaps directory", e.record_start, e.name);
    if n != name.count return problem(.INVALID, "local/central name mismatch", e.record_start, e.name);

    text, failure := region(a, e.record_start + 30, n);
    if failure.kind != .NONE return failure;

    equal := cast(string) text.bytes == cast(string) name;

    free_archive(*text);
    if !equal return problem(.INVALID, "local/central name mismatch", e.record_start, e.name);

    e.offset = e.record_start + 30 + n + x;
    if !fits(central, e.offset, e.compressed_size) return problem(.INVALID, "entry overlaps directory", e.offset, e.name);

    e.record_end = e.offset + e.compressed_size;
    if e.method == .STORED && e.size != e.compressed_size return problem(.INVALID, "stored size mismatch", e.offset, e.name);

    error = zip_local_sizes(a, e, h, n, x, flags);
    if error.kind != .NONE return error;

    if !(flags & 8) {
        if u32le(h, 14) != e.crc return problem(.INVALID, "local CRC mismatch", e.record_start, e.name);
        if u32le(h, 18) != 0xffffffff && cast(s64) u32le(h, 18) != e.compressed_size return problem(.INVALID, "local size mismatch", e.record_start, e.name);
        if u32le(h, 22) != 0xffffffff && cast(s64) u32le(h, 22) != e.size return problem(.INVALID, "local size mismatch", e.record_start, e.name);
    } else {
        descriptor: [24] u8;
        length := ifx wide then 20 else 12;

        error = fetch_at(a, e.record_end, slice(descriptor, 0, 4));
        if error.kind != .NONE return error;

        skip := ifx u32le(descriptor, 0) == 0x08074b50 then 4 else 0;
        if !fits(central, e.record_end, length + skip) return problem(.INVALID, "truncated data descriptor", e.record_end, e.name);

        error = fetch_at(a, e.record_end, slice(descriptor, 0, length + skip));
        if error.kind != .NONE return error;

        packed := ifx wide then u64le(descriptor, skip + 4) else cast(u64) u32le(descriptor, skip + 4);
        size := ifx wide then u64le(descriptor, skip + 12) else cast(u64) u32le(descriptor, skip + 8);

        if u32le(descriptor, skip) != e.crc || packed != cast(u64) e.compressed_size || size != cast(u64) e.size return problem(.INVALID, "invalid data descriptor", e.record_end, e.name);

        e.record_end += length + skip;
    }

    return .{};
}

zip_name :: (bytes: [] u8, utf8: bool) -> string, bool {
    if utf8 {
        if !valid_utf8(bytes) return "", false;
        return copy_string(cast(string) bytes), true;
    }

    out: [..] u8;

    for bytes {
        if it < 128 array_add(*out, it);
        else {
            c := CP437[it - 128];
            if c < 2048 array_add(*out, cast(u8) (0xc0 | (c >> 6)));
            else {
                array_add(*out, cast(u8) (0xe0 | (c >> 12)));
                array_add(*out, cast(u8) (0x80 | ((c >> 6) & 63)));
            }

            array_add(*out, cast(u8) (0x80 | (c & 63)));
        }
    }

    return .{ out.count, out.data }, true;
}

valid_utf8 :: (b: [] u8) -> bool {
    i: s64;
    while i < b.count {
        _, n, status := Unicode.character_utf8_to_utf32(b.data + i, b.count - i, true);
        if status != .CONVERSION_OK return false;
        i += n;
    }

    return true;
}

CP437 :: u16.[
    0xc7, 0xfc, 0xe9, 0xe2, 0xe4, 0xe0, 0xe5, 0xe7,
    0xea, 0xeb, 0xe8, 0xef, 0xee, 0xec, 0xc4, 0xc5,
    0xc9, 0xe6, 0xc6, 0xf4, 0xf6, 0xf2, 0xfb, 0xf9,
    0xff, 0xd6, 0xdc, 0xa2, 0xa3, 0xa5, 0x20a7, 0x192,
    0xe1, 0xed, 0xf3, 0xfa, 0xf1, 0xd1, 0xaa, 0xba,
    0xbf, 0x2310, 0xac, 0xbd, 0xbc, 0xa1, 0xab, 0xbb,
    0x2591, 0x2592, 0x2593, 0x2502, 0x2524, 0x2561, 0x2562, 0x2556,
    0x2555, 0x2563, 0x2551, 0x2557, 0x255d, 0x255c, 0x255b, 0x2510,
    0x2514, 0x2534, 0x252c, 0x251c, 0x2500, 0x253c, 0x255e, 0x255f,
    0x255a, 0x2554, 0x2569, 0x2566, 0x2560, 0x2550, 0x256c, 0x2567,
    0x2568, 0x2564, 0x2565, 0x2559, 0x2558, 0x2552, 0x2553, 0x256b,
    0x256a, 0x2518, 0x250c, 0x2588, 0x2584, 0x258c, 0x2590, 0x2580,
    0x3b1, 0xdf, 0x393, 0x3c0, 0x3a3, 0x3c3, 0xb5, 0x3c4,
    0x3a6, 0x398, 0x3a9, 0x3b4, 0x221e, 0x3c6, 0x3b5, 0x2229,
    0x2261, 0xb1, 0x2265, 0x2264, 0x2320, 0x2321, 0xf7, 0x2248,
    0xb0, 0x2219, 0xb7, 0x221a, 0x207f, 0xb2, 0x25a0, 0xa0,
];

zip_central_entry :: (a: *Archive, d: [] u8, at: s64, start: u64, base: s64) -> Archive_Entry, s64, Archive_Error {
    error: Archive_Error;

    if !fits(d.count, at, 46) || u32le(d, at) != 0x02014b50 return .{}, 0, problem(.INVALID, "invalid central header", cast(s64) start + at);

    flags := u16le(d, at + 8);
    method := u16le(d, at + 10);
    n := cast(s64) u16le(d, at + 28);
    x := cast(s64) u16le(d, at + 30);
    c := cast(s64) u16le(d, at + 32);

    if !fits(d.count, at + 46, n + x + c) return .{}, 0, problem(.INVALID, "truncated central header", cast(s64) start + at);

    raw_name := slice(d, at + 46, n);
    name, valid := zip_name(raw_name, (flags & 0x800) != 0);

    if !valid return .{}, 0, problem(.INVALID, "invalid UTF-8 name", cast(s64) start + at);

    e: Archive_Entry;
    e.name = name;
    e.zip = true;
    e.crc = u32le(d, at + 16);
    e.compressed_size = cast(s64) u32le(d, at + 20);
    e.size = cast(s64) u32le(d, at + 24);
    local := cast(u64) u32le(d, at + 42);
    disk := cast(u32) u16le(d, at + 34);

    if flags & ~cast(u16) 0x080e error = problem(.UNSUPPORTED, "encrypted or unsupported ZIP flags", cast(s64) start + at, name);
    else if method != 0 && method != 8 error = problem(.UNSUPPORTED, "unsupported ZIP compression", cast(s64) start + at, name);

    e.method = cast(Archive_Method) method;
    need_size := e.size == 0xffffffff;
    need_packed := e.compressed_size == 0xffffffff;
    need_local := local == 0xffffffff;
    need_disk := disk == 65535;
    extra := slice(d, at + 46 + n, x);
    p: s64;
    found := false;

    while p < extra.count && error.kind == .NONE {
        if !fits(extra.count, p, 4) {
            error = problem(.INVALID, "truncated ZIP extra field", cast(s64) start + at, name);
            break;
        }

        tag := u16le(extra, p);
        length := cast(s64) u16le(extra, p + 2);

        p += 4;
        if !fits(extra.count, p, length) {
            error = problem(.INVALID, "truncated ZIP extra field", cast(s64) start + at, name);
            break;
        }

        if tag == 1 {
            values := slice(extra, p, length);
            q: s64;

            for which: 0..3 {
                needed := ifx which == 0 then need_size else ifx which == 1 then need_packed else ifx which == 2 then need_local else need_disk;
                if !needed continue;

                width := ifx which == 3 then 4 else 8;
                if !fits(values.count, q, width) {
                    error = problem(.INVALID, "short ZIP64 extra field", cast(s64) start + at, name);
                    break;
                }

                value := ifx width == 8 then u64le(values, q) else cast(u64) u32le(values, q);
                if value > 0x7fffffffffffffff {
                    error = problem(.LIMIT, "ZIP64 integer too large", cast(s64) start + at, name);
                    break;
                }

                if which == 0 e.size = cast(s64) value;
                if which == 1 e.compressed_size = cast(s64) value;
                if which == 2 local = value;
                if which == 3 disk = cast(u32) value;

                q += width;
            }

            found = true;
        }

        p += length;
    }

    if (need_size || need_packed || need_local || need_disk) && !found && error.kind == .NONE error = problem(.INVALID, "missing ZIP64 extra field", cast(s64) start + at, name);
    if disk && error.kind == .NONE error = problem(.UNSUPPORTED, "multi-disk ZIP entry", cast(s64) start + at, name);

    if error.kind == .NONE {
        if local > cast(u64) (a.source.size - base) error = problem(.INVALID, "local offset outside ZIP", cast(s64) start + at, name);
        else {
            e.record_start = cast(s64) local + base;
            mode := (u32le(d, at + 38) >> 16) & 0xf000;
            if mode == 0xa000 e.kind = .SYMLINK;
            else if mode == 0x4000 || (name.count && name[name.count - 1] == #char "/") e.kind = .DIRECTORY;
            else if mode && mode != 0x8000 e.kind = .OTHER;

            error = zip_local(a, *e, flags, raw_name, cast(s64) start, need_size || need_packed);
        }
    }

    if error.kind != .NONE {
        if !error.entry.count error.entry = copy_string(name);
        free(name);
        return .{}, 0, error;
    }

    return e, 46 + n + x + c, .{};
}

zip_local_sizes :: (a: *Archive, e: *Archive_Entry, h: [] u8, n: s64, x: s64, flags: u16) -> Archive_Error {
    extra, error := region(a, e.record_start + 30 + n, x);
    if error.kind != .NONE return error;

    defer free_archive(*extra);
    at: s64;
    found := false;
    packed := cast(u64) u32le(h, 18);
    size := cast(u64) u32le(h, 22);
    need_size := size == 0xffffffff;
    need_packed := packed == 0xffffffff;

    while at < extra.bytes.count {
        if !fits(extra.bytes.count, at, 4) return problem(.INVALID, "truncated local extra field", e.record_start, e.name);

        tag := u16le(extra.bytes, at);
        length := cast(s64) u16le(extra.bytes, at + 2);

        at += 4;
        if !fits(extra.bytes.count, at, length) return problem(.INVALID, "truncated local extra field", e.record_start, e.name);

        if tag == 1 {
            if found return problem(.INVALID, "duplicate local ZIP64 field", e.record_start, e.name);

            found = true;
            p := at;

            if need_size {
                if p + 8 > at + length return problem(.INVALID, "short local ZIP64 field", e.record_start, e.name);
                size = u64le(extra.bytes, p);
                p += 8;
            }

            if need_packed {
                if p + 8 > at + length return problem(.INVALID, "short local ZIP64 field", e.record_start, e.name);
                packed = u64le(extra.bytes, p);
            }
        }

        at += length;
    }

    if (need_size || need_packed) && !found return problem(.INVALID, "missing local ZIP64 field", e.record_start, e.name);
    if !(flags & 8) && (size != cast(u64) e.size || packed != cast(u64) e.compressed_size) return problem(.INVALID, "local ZIP64 size mismatch", e.record_start, e.name);

    return .{};
}
