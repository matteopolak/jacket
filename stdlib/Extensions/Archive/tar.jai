#scope_module

// Pax keywords from an extended header. A keyword is unset, set, or (in a local `x` header only)
// deleted by an empty value, which hides the global value and falls back to the header block.
Tar_Overrides :: struct {
    path: string;
    link: string;
    size: s64 = -1;
    mtime: float64;
    has_mtime: bool;
    path_deleted: bool;
    link_deleted: bool;
    size_deleted: bool;
    mtime_deleted: bool;
}

has_overrides :: (p: Tar_Overrides) -> bool {
    return p.path.count || p.link.count || p.size >= 0 || p.has_mtime || p.path_deleted || p.link_deleted || p.size_deleted || p.mtime_deleted;
}

clear_overrides :: (p: *Tar_Overrides) {
    free(p.path);
    free(p.link);
    p.* = .{};
}

tar_index :: (a: *Archive) -> Archive_Error {
    at: s64;
    pending: Tar_Overrides;
    global: Tar_Overrides;

    defer clear_overrides(*pending);
    defer clear_overrides(*global);
    extensions: s64;
    metadata: s64;

    while at < a.source.size {
        h: [512] u8;
        error := fetch_at(a, at, h);

        if error.kind != .NONE return error;

        zero := true;

        for h {
            if it {
                zero = false;
                break;
            }
        }

        if zero {
            if has_overrides(pending) return problem(.INVALID, "orphan TAR extended header", at);
            return .{};
        }

        checksum, valid := tar_number(slice(h, 148, 8));
        sum: s64;
        signed_sum: s64;

        for h {
            if it_index >= 148 && it_index < 156 {
                sum += 32;
                signed_sum += 32;
            } else {
                sum += cast(s64) it;
                signed_sum += cast(s64) cast,trunc(s8) it;
            }
        }

        if !valid || (checksum != sum && checksum != signed_sum) return problem(.CHECKSUM, "TAR header checksum", at);

        size, size_ok := tar_number(slice(h, 124, 12));
        mtime, time_ok := tar_number(slice(h, 136, 12));

        if !size_ok || !time_ok || size < 0 return problem(.INVALID, "invalid TAR number", at);

        kind := h[156];
        if kind == #char "x" || kind == #char "g" || kind == #char "L" || kind == #char "K" {
            if size > a.limits.max_metadata - metadata || extensions >= a.limits.max_entries return problem(.LIMIT, "TAR extension limit", at);

            metadata += size;
            extensions += 1;
            payload, failure := region(a, at + 512, size);
            if failure.kind != .NONE return failure;

            if kind == #char "L" || kind == #char "K" {
                text := tar_text(payload.bytes);

                if kind == #char "L" {
                    free(pending.path);
                    pending.path = copy_string(text);
                } else {
                    free(pending.link);
                    pending.link = copy_string(text);
                }
            } else {
                target := ifx kind == #char "g" then *global else *pending;
                error = parse_pax(payload.bytes, target, at + 512, kind == #char "x");
            }

            free_archive(*payload);
            if error.kind != .NONE return error;
        } else {
            e := tar_entry(h, pending, global, size, mtime);
            e.kind = ifx kind == 0 || kind == #char "0" then .FILE else ifx kind == #char "5" then .DIRECTORY else ifx kind == #char "2" then .SYMLINK else ifx kind == #char "1" then .HARDLINK else .OTHER;
            e.offset = at + 512;
            e.record_start = at;
            if kind == #char "S" error = problem(.UNSUPPORTED, "GNU sparse TAR", at, e.name);
            else if !fits(a.source.size, e.offset, e.size) error = problem(.INVALID, "truncated TAR payload", at, e.name);
            if e.kind == .DIRECTORY || e.kind == .SYMLINK || e.kind == .HARDLINK || kind == #char "3" || kind == #char "4" || kind == #char "6" e.size = 0;

            e.compressed_size = e.size;
            size = e.size;
            if error.kind == .NONE error = admit(a, e);

            if error.kind != .NONE {
                free(e.name);
                free(e.link);
                return error;
            }

            clear_overrides(*pending);
        }

        if size > a.source.size - at - 512 return problem(.INVALID, "truncated TAR payload", at);

        padding := (512 - size % 512) % 512;
        if !fits(a.source.size, at + 512, size + padding) return problem(.INVALID, "truncated TAR padding", at);

        at += 512 + size + padding;
    }

    if has_overrides(pending) return problem(.INVALID, "orphan TAR extended header", at);

    return .{};
}

tar_text :: (b: [] u8) -> string {
    n: s64;
    while n < b.count && b[n] != 0 n += 1;
    return .{ n, b.data };
}

tar_number :: (b: [] u8) -> s64, bool {
    n: s64;

    if b[0] & 128 {
        negative := (b[0] & 64) != 0;

        // Sign extension permits long base-256 fields without overflowing the accumulator.
        for b {
            v := ifx it_index == 0 then it & 127 else it;

            if it_index < b.count - 8 {
                if v != (ifx negative then (ifx it_index == 0 then 127 else 255) else 0) return 0, false;
            } else {
                n = cast,trunc(s64) (((cast,trunc(u64) n) << 8) | (cast(u64) v));
            }
        }

        if negative != (n < 0) return 0, false;

        return n, true;
    }

    begin: s64;
    while begin < b.count && (b[begin] == 0 || b[begin] == 32) begin += 1;

    end := begin;
    while end < b.count && b[end] >= #char "0" && b[end] <= #char "7" {
        if n > (0x7fffffffffffffff - cast(s64) (b[end] - #char "0")) / 8 return 0, false;
        n = n * 8 + cast(s64) (b[end] - #char "0");
        end += 1;
    }

    for i: end..b.count - 1 {
        if b[i] != 0 && b[i] != 32 return 0, false;
    }

    return n, true;
}

decimal :: (b: [] u8) -> s64, bool {
    if !b.count return 0, false;

    n: s64;

    for b {
        if it < #char "0" || it > #char "9" return 0, false;
        digit := cast(s64) (it - #char "0");
        if n > (0x7fffffffffffffff - digit) / 10 return 0, false;
        n = n * 10 + digit;
    }

    return n, true;
}

// Applies the records of one extended header. An empty value deletes the keyword: a global header
// forgets its earlier value, a local one also hides the global value for this entry.
parse_pax :: (b: [] u8, p: *Tar_Overrides, offset: s64, local: bool) -> Archive_Error {
    at: s64;
    while at < b.count {
        space := at;
        while space < b.count && b[space] != 32 space += 1;

        length, ok := decimal(slice(b, at, space - at));
        if !ok || length <= space - at + 2 || !fits(b.count, at, length) || b[at + length - 1] != 10 return problem(.INVALID, "invalid pax record length", offset + at);

        equal := space + 1;
        while equal < at + length - 1 && b[equal] != #char "=" equal += 1;
        if equal == at + length - 1 return problem(.INVALID, "invalid pax key", offset + at);

        key := cast(string) slice(b, space + 1, equal - space - 1);
        value := slice(b, equal + 1, at + length - equal - 2);

        if key == "path" || key == "linkpath" {
            target := ifx key == "path" then *p.path else *p.link;
            deleted := ifx key == "path" then *p.path_deleted else *p.link_deleted;

            if !value.count {
                free(target.*);
                target.* = .{};
                deleted.* = local;
                at += length;
                continue;
            }

            for value {
                if it == 0 return problem(.INVALID, "NUL in pax path", offset + equal + 1);
            }

            if !valid_utf8(value) return problem(.INVALID, "invalid pax UTF-8", offset + equal + 1);

            free(target.*);
            target.* = copy_string(cast(string) value);
            deleted.* = false;
        } else if key == "size" {
            if !value.count {
                p.size = -1;
                p.size_deleted = local;
                at += length;
                continue;
            }

            number, valid := decimal(value);
            if !valid return problem(.INVALID, "invalid pax size", offset + equal + 1);

            p.size = number;
            p.size_deleted = false;
        } else if key == "mtime" {
            if !value.count {
                p.mtime = 0;
                p.has_mtime = false;
                p.mtime_deleted = local;
                at += length;
                continue;
            }

            value_text := cast(string) value;
            number, valid, rest := string_to_float64(value_text);

            if !valid || rest.count || !pax_time_syntax(value) || number != number || number > 9.22e18 || number < -9.22e18 return problem(.INVALID, "invalid pax mtime", offset + equal + 1);

            p.mtime = number;
            p.has_mtime = true;
            p.mtime_deleted = false;
        } else if key.count >= 10 && slice(key, 0, 10) == "GNU.sparse" return problem(.UNSUPPORTED, "pax sparse TAR", offset + at);

        at += length;
    }

    return .{};
}

pax_time_syntax :: (b: [] u8) -> bool {
    i: s64;
    if b.count && b[0] == #char "-" i += 1;

    begin := i;

    while i < b.count && b[i] >= #char "0" && b[i] <= #char "9" i += 1;
    if i == begin return false;

    if i < b.count && b[i] == #char "." {
        i += 1;
        begin = i;
        while i < b.count && b[i] >= #char "0" && b[i] <= #char "9" i += 1;
        if i == begin return false;
    }

    return i == b.count;
}

tar_entry :: (h: [] u8, pending: Tar_Overrides, global: Tar_Overrides, size: s64, mtime: s64) -> Archive_Entry {
    e: Archive_Entry;
    e.name = copy_string(tar_text(slice(h, 0, 100)));
    e.link = copy_string(tar_text(slice(h, 157, 100)));

    // GNU headers use the prefix area for other fields.
    if cast(string) slice(h, 257, 6) == "ustar\0" {
        prefix := tar_text(slice(h, 345, 155));
        if prefix.count {
            joined := sprint("%/%", prefix, e.name);
            free(e.name);
            e.name = joined;
        }
    }

    e.size = size;
    e.mtime = cast(float64) mtime;

    // A local keyword wins over the global one, and a local deletion leaves the header block's value.
    if !pending.path_deleted {
        text := ifx pending.path.count then pending.path else global.path;

        if text.count {
            free(e.name);
            e.name = copy_string(text);
        }
    }

    if !pending.link_deleted {
        text := ifx pending.link.count then pending.link else global.link;

        if text.count {
            free(e.link);
            e.link = copy_string(text);
        }
    }

    if !pending.size_deleted {
        value := ifx pending.size >= 0 then pending.size else global.size;
        if value >= 0 e.size = value;
    }

    if !pending.mtime_deleted {
        if pending.has_mtime e.mtime = pending.mtime;
        else if global.has_mtime e.mtime = global.mtime;
    }

    return e;
}
