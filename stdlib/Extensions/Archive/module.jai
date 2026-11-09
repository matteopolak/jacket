// Reads and writes ZIP and TAR archives within explicit size limits. Entry metadata is owned by the
// [Archive]; payloads are read on demand. Input bytes (or a source) must outlive the archive.
//
// ```jai
// #import "Extensions/Archive";
//
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hi" }];
// bytes, error := encode_archive(items, .ZIP);
// assert(error.kind == .NONE);
// defer free(bytes.data);
//
// a, e := read_archive(bytes, .ZIP);
// assert(e.kind == .NONE);
// defer free_archive(*a);
// assert(a.entries.count == 1 && a.entries[0].name == "a.txt");
// ```

// The container format of an archive.
Archive_Format :: enum {
    ZIP;
    TAR;
    TAR_GZIP; // a TAR stream compressed with gzip
}

// What an entry is.
Archive_Kind :: enum {
    FILE;
    DIRECTORY;
    SYMLINK;
    HARDLINK;
    OTHER; // a device, FIFO or other special record; readable, never written or extracted
}

// How a ZIP entry's payload is stored.
Archive_Method :: enum u16 {
    STORED = 0;
    DEFLATE = 8;
}

// Why an archive operation failed.
Archive_Error_Kind :: enum {
    NONE;
    INVALID; // malformed or inconsistent structure
    UNSUPPORTED; // a valid feature this module does not handle
    LIMIT; // an [Archive_Limits] bound was exceeded
    CHECKSUM; // size or CRC mismatch
    PATH; // an unsafe path while extracting
    DUPLICATE; // the same name twice while extracting
    IO; // a source, writer or file operation failed
}

// The outcome of an operation; `kind` is `.NONE` on success. Release `entry` with [free_archive].
Archive_Error :: struct {
    kind: Archive_Error_Kind;
    message: string; // static
    offset: s64; // byte position in the archive where it went wrong, when known
    entry: string; // owned; free_archive releases it
}

// Bounds applied while indexing, reading, writing and extracting, to contain hostile input.
// A negative value (or a `max_ratio` of 0 or less) makes the operation fail with `.INVALID`.
Archive_Limits :: struct {
    max_archive: s64 = 1 << 30; // archive size in bytes
    max_metadata: s64 = 16 << 20; // total bytes of names, links and headers
    max_entries: s64 = 100000;
    max_entry: s64 = 256 << 20; // uncompressed size of one entry
    max_total: s64 = 1 << 30; // uncompressed size of all entries
    max_ratio: s64 = 1000; // largest uncompressed-to-stored size ratio of an entry
}

// A seekable byte source for [read_archive]. A read-at callback may return short reads. Its source
// must stay alive and unchanged.
Archive_Source :: struct {
    data: *void; // passed back to `read_at`
    size: s64; // total bytes available
    read_at: (data: *void, offset: s64, buffer: [] u8) -> s64, string;
}

// One indexed archive entry. The strings are owned by the [Archive] that holds the entry.
Archive_Entry :: struct {
    name: string;
    link: string; // the target of a symlink or hardlink
    kind: Archive_Kind;
    size: s64; // uncompressed size
    compressed_size: s64; // stored size
    mtime: float64; // seconds since the Unix epoch
    method: Archive_Method;
    crc: u32; // CRC-32 of the uncompressed bytes (ZIP)
    offset: s64; // where the payload starts in the archive
    record_start: s64; // where the entry's whole record starts
    record_end: s64;
    zip: bool;
}

// An open archive: the entry index plus what is needed to read payloads. Release it with
// [free_archive].
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

// The bytes of one entry from [read_entry]. When `owned` they were allocated and [free_archive]
// releases them; otherwise they borrow the archive's input.
Archive_Data :: struct {
    bytes: [] u8;
    owned: bool;
}

// An entry to write with [write_archive] or [encode_archive]. Nothing is copied: `name`, `data` and
// `link` must stay valid until the call returns.
Archive_Item :: struct {
    name: string;
    data: [] u8;
    kind: Archive_Kind;
    link: string; // the target of a symlink or hardlink (TAR only)
    mtime: s64; // seconds since the Unix epoch
    method: Archive_Method; // ZIP only
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
// Example: error := copy_entry(*a, 0, sink);
copy_entry :: (a: *Archive, index: s64, writer: Archive_Writer) -> Archive_Error {
    if index < 0 || index >= a.entries.count return problem(.INVALID, "entry index out of range");

    e := a.entries[index];
    if e.method == .DEFLATE {
        value, error := read_entry(a, index);
        if error.kind != .NONE return error;

        defer free_archive(*value);
        if !writer.write_proc(writer.data, value.bytes) return problem(.IO, "entry write failed", e.offset, e.name);

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
        if !writer.write_proc(writer.data, chunk) return problem(.IO, "entry write failed", e.offset + at, e.name);

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

// Receives output bytes; returns false when it cannot take them all. `data` is the sink's own state.
Archive_Write_Proc :: #type (data: *void, bytes: [] u8) -> bool;

// A byte sink for `write_archive` and `copy_entry`. It owns nothing and works on every target, wasm
// included. To write to a file or socket, point `data` at it and forward to its own write, as
// `forward` does for an `Extensions/Io` writer.
//
// ```jai,ignore
// forward :: (data: *void, bytes: [] u8) -> bool {
//     _, error := Io.write((cast(*Io.Writer) data).*, bytes);
//     return error == .NONE;
// }
//
// io_writer := Io.writer_of(*file);
// error := write_archive(Archive_Writer.{ data = *io_writer, write_proc = forward }, items, .ZIP);
// ```
Archive_Writer :: struct {
    data: *void;
    write_proc: Archive_Write_Proc;
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
