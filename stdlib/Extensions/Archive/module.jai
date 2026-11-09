// Bounded ZIP and TAR containers. Metadata is owned; byte-view input must outlive the archive.
// Example: a, e := read_archive(bytes, .ZIP); defer free_archive(*a);
Archive_Format :: enum {
    ZIP;
    TAR;
    TAR_GZIP;
}

Archive_Kind :: enum {
    FILE;
    DIRECTORY;
    SYMLINK;
    HARDLINK;
    OTHER;
}

Archive_Method :: enum u16 {
    STORED = 0;
    DEFLATE = 8;
}

Archive_Error_Kind :: enum {
    NONE;
    INVALID;
    UNSUPPORTED;
    LIMIT;
    CHECKSUM;
    PATH;
    DUPLICATE;
    IO;
}

Archive_Error :: struct {
    kind: Archive_Error_Kind;
    message: string; // static
    offset: s64;
    entry: string; // owned; free_archive releases it
}

Archive_Limits :: struct {
    max_archive: s64 = 1 << 30;
    max_metadata: s64 = 16 << 20;
    max_entries: s64 = 100000;
    max_entry: s64 = 256 << 20;
    max_total: s64 = 1 << 30;
    max_ratio: s64 = 1000;
}

// A read-at callback may return short reads. Its source must stay alive and unchanged.
Archive_Source :: struct {
    data: *void;
    size: s64;
    read_at: (data: *void, offset: s64, buffer: [] u8) -> s64, string;
}

Archive_Entry :: struct {
    name: string;
    link: string;
    kind: Archive_Kind;
    size: s64;
    compressed_size: s64;
    mtime: float64;
    method: Archive_Method;
    crc: u32;
    offset: s64;
    record_start: s64;
    record_end: s64;
    zip: bool;
}

Archive :: struct {
    entries: [..] Archive_Entry;
    limits: Archive_Limits;
    // Internal; do not change these fields or entries while using the archive.
    bytes: [] u8;
    source: Archive_Source;
    owned: [] u8;
    format: Archive_Format;
    metadata: s64;
    total: s64;
}

Archive_Data :: struct {
    bytes: [] u8;
    owned: bool;
}

Archive_Item :: struct {
    name: string;
    data: [] u8;
    kind: Archive_Kind;
    link: string;
    mtime: s64;
    method: Archive_Method;
}

// Reads an index without copying stored payloads. Example: a, e := read_archive(bytes, .TAR);
read_archive :: (bytes: [] u8, format: Archive_Format, limits := Archive_Limits.{}) -> Archive, Archive_Error {
    a: Archive;
    a.bytes = bytes;
    a.source.size = bytes.count;
    a.format = format;
    a.limits = limits;

    return open_index(a);
}

// Indexes a seekable source with bounded metadata reads. Example: a, e := read_archive(source, .ZIP);
read_archive :: (source: Archive_Source, format: Archive_Format, limits := Archive_Limits.{}) -> Archive, Archive_Error {
    a: Archive;
    a.source = source;
    a.format = format;
    a.limits = limits;

    return open_index(a);
}

// Returns a borrowed stored payload or an owned inflated buffer, verifying ZIP CRC first.
// Example: d, e := read_entry(*a, 0); defer free_archive(*d);
read_entry :: (a: *Archive, index: s64) -> Archive_Data, Archive_Error {
    if index < 0 || index >= a.entries.count return .{}, problem(.INVALID, "entry index out of range");

    e := a.entries[index];
    input, error := region(a, e.offset, e.compressed_size);

    if error.kind != .NONE {
        error.entry = copy_string(e.name);
        return .{}, error;
    }

    if e.method == .DEFLATE {
        output, ok, message := Deflate.inflate_complete(input.bytes, .RAW, e.size);
        free_archive(*input);
        if !ok return .{}, problem(.INVALID, message, e.offset, e.name);
        input = .{ output, true };
    }

    if input.bytes.count != e.size || (e.zip && Deflate.crc32(input.bytes) != e.crc) {
        free_archive(*input);
        return .{}, problem(.CHECKSUM, "entry size or CRC mismatch", e.offset, e.name);
    }

    return input, .{};
}

// Copies stored data in 64 KiB pieces; deflate uses one bounded entry buffer.
// Example: error := copy_entry(*a, 0, Io.writer_of(*file));
copy_entry :: (a: *Archive, index: s64, writer: Io.Writer) -> Archive_Error {
    if index < 0 || index >= a.entries.count return problem(.INVALID, "entry index out of range");

    e := a.entries[index];
    if e.method == .DEFLATE {
        value, error := read_entry(a, index);
        if error.kind != .NONE return error;

        defer free_archive(*value);
        _, failure := Io.write(writer, value.bytes);
        if failure != .NONE return problem(.IO, "entry write failed", e.offset, e.name);

        return .{};
    }

    buffer: [65536] u8;
    at: s64;
    crc: u32;

    while at < e.size {
        n := min(buffer.count, e.size - at);
        chunk := slice(buffer, 0, n);
        error := fetch_at(a, e.offset + at, chunk);

        if error.kind != .NONE {
            error.entry = copy_string(e.name);
            return error;
        }

        crc = Deflate.crc32(chunk, crc);
        _, failure := Io.write(writer, chunk);
        if failure != .NONE return problem(.IO, "entry write failed", e.offset + at, e.name);

        at += n;
    }

    if e.zip && crc != e.crc return problem(.CHECKSUM, "entry CRC mismatch", e.offset, e.name);

    return .{};
}

// Releases index metadata and gzip backing; never frees borrowed input. Example: free_archive(*a);
free_archive :: (a: *Archive) {
    for a.entries {
        free(it.name);
        free(it.link);
    }

    array_free(a.entries);
    if a.owned.data free(a.owned.data);

    a.* = .{};
}

// Releases an entry buffer when owned. Example: free_archive(*d);
free_archive :: (data: *Archive_Data) {
    if data.owned && data.bytes.data free(data.bytes.data);
    data.* = .{};
}

// Releases the error's entry name. Example: free_archive(*error);
free_archive :: (error: *Archive_Error) {
    if error.entry.data free(error.entry);
    error.* = .{};
}

#load "zip.jai";
#load "tar.jai";
#load "write.jai";
#load "extract.jai";

#scope_module
#import "Basic";
#import "String";
#import "Sort";
Deflate :: #import "Extensions/Deflate";
Io :: #import "Extensions/Io";
Unicode :: #import "Unicode";
Map :: #import "Extensions/Hash_Map";

problem :: (kind: Archive_Error_Kind, message: string, offset: s64 = 0, entry := "") -> Archive_Error {
    owned_entry: string;
    if entry.count owned_entry = copy_string(entry);
    return .{ kind, message, offset, owned_entry };
}

valid_limits :: (l: Archive_Limits) -> bool {
    return l.max_archive >= 0 && l.max_metadata >= 0 && l.max_entries >= 0 && l.max_entry >= 0 && l.max_total >= 0 && l.max_ratio > 0;
}

open_index :: (initial: Archive) -> Archive, Archive_Error {
    a := initial;
    error: Archive_Error;

    if !valid_limits(a.limits) || a.source.size < 0 || a.source.size > a.limits.max_archive {
        return .{}, problem(.LIMIT, "invalid limits or archive too large");
    }

    if !a.bytes.data && a.source.size && !a.source.read_at return .{}, problem(.IO, "missing read-at callback");

    if a.format == .TAR_GZIP {
        input, failure := region(*a, 0, a.source.size);
        if failure.kind != .NONE return .{}, failure;

        cap := min(a.limits.max_total, a.limits.max_archive);
        if a.source.size <= cap / a.limits.max_ratio cap = a.source.size * a.limits.max_ratio;

        data, ok, message := Deflate.inflate_complete(input.bytes, .GZIP, cap);

        free_archive(*input);
        if !ok return .{}, problem(.INVALID, message);

        a.owned = data;
        a.bytes = data;
        a.source = .{ size = data.count };
    }

    if a.format == .ZIP error = zip_index(*a);
    else error = tar_index(*a);

    if error.kind != .NONE {
        free_archive(*a);
        return .{}, error;
    }

    return a, .{};
}

fits :: (size: s64, offset: s64, count: s64) -> bool {
    return offset >= 0 && count >= 0 && offset <= size && count <= size - offset;
}

fetch_at :: (a: *Archive, offset: s64, output: [] u8) -> Archive_Error {
    if !fits(a.source.size, offset, output.count) return problem(.INVALID, "truncated archive", offset);

    if a.bytes.data {
        if output.count memcpy(output.data, a.bytes.data + offset, output.count);
        return .{};
    }

    at: s64;
    while at < output.count {
        n, message := a.source.read_at(a.source.data, offset + at, slice(output, at, output.count - at));
        if message.count || n <= 0 || n > output.count - at return problem(.IO, "source read failed", offset + at);
        at += n;
    }

    return .{};
}

region :: (a: *Archive, offset: s64, count: s64) -> Archive_Data, Archive_Error {
    if !fits(a.source.size, offset, count) return .{}, problem(.INVALID, "truncated archive", offset);
    if a.bytes.data return .{ .{ count, a.bytes.data + offset }, false }, .{};

    data: [] u8 = .{ count, alloc(max(count, 1)) };
    error := fetch_at(a, offset, data);

    if error.kind != .NONE {
        free(data.data);
        return .{}, error;
    }

    return .{ data, true }, .{};
}

u16le :: (b: [] u8, p: s64) -> u16 {
    lo: u16 = b[p];
    hi: u16 = b[p + 1];
    return lo | (hi << 8);
}

u32le :: (b: [] u8, p: s64) -> u32 {
    lo: u32 = u16le(b, p);
    hi: u32 = u16le(b, p + 2);
    return lo | (hi << 16);
}

u64le :: (b: [] u8, p: s64) -> u64 {
    lo: u64 = u32le(b, p);
    hi: u64 = u32le(b, p + 4);
    return lo | (hi << 32);
}

admit :: (a: *Archive, e: Archive_Entry) -> Archive_Error {
    if a.entries.count >= a.limits.max_entries || e.size < 0 || e.size > a.limits.max_entry || e.size > a.limits.max_total - a.total return problem(.LIMIT, "entry count or size limit", e.offset, e.name);
    if e.compressed_size < 0 || (e.size > 0 && (e.compressed_size == 0 || (e.size - 1) / max(e.compressed_size, 1) >= a.limits.max_ratio)) return problem(.LIMIT, "compression ratio limit", e.offset, e.name);

    cost := e.name.count + e.link.count;
    if cost > a.limits.max_metadata - a.metadata return problem(.LIMIT, "metadata limit", e.offset, e.name);

    a.total += e.size;
    a.metadata += cost;
    array_add(*a.entries, e);

    return .{};
}

slice :: (b: [] u8, start: s64, count: s64) -> [] u8 {
    return .{ count, b.data + start };
}
