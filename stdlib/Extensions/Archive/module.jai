// Reads and writes ZIP and TAR archives within explicit size limits.
//
// - [read_archive] indexes an archive from bytes or a seekable [Archive_Source]; [read_entry] and
//   [copy_entry] read one entry's payload on demand, checking its size and CRC.
// - [encode_archive] writes to a new buffer and [write_archive] to any [Archive_Writer].
// - [extract] unpacks files and directories into a folder without following or creating links.
//
// Every call takes [Archive_Limits], so a hostile archive fails with `.LIMIT` instead of exhausting
// memory. Entry names are owned by the [Archive]; the input bytes or source must outlive it. Free
// archives, owned payloads and errors with [free_archive].
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
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hi" }];
// bytes, error := encode_archive(items, .TAR_GZIP);
// assert(error.kind == .NONE);
// defer free(bytes.data);
// assert(bytes[0] == 0x1f && bytes[1] == 0x8b); // the gzip magic
// ```
Archive_Format :: enum {
    ZIP;
    TAR;
    TAR_GZIP; // a TAR stream compressed with gzip
}

// What an entry is. ZIP archives hold only files and directories; TAR also holds links.
//
// ```jai
// items := Archive_Item.[
//     .{ name = "docs/", kind = .DIRECTORY },
//     .{ name = "latest", kind = .SYMLINK, link = "docs/" },
// ];
// bytes, error := encode_archive(items, .TAR);
// assert(error.kind == .NONE);
// defer free(bytes.data);
//
// archive, read_error := read_archive(bytes, .TAR);
// defer free_archive(*archive);
// assert(archive.entries[1].kind == .SYMLINK && archive.entries[1].link == "docs/");
// ```
Archive_Kind :: enum {
    FILE;
    DIRECTORY;
    SYMLINK;
    HARDLINK;
    OTHER; // a device, FIFO or other special record; readable, never written or extracted
}

// How a ZIP entry's payload is stored. TAR ignores it; compress a whole TAR with `.TAR_GZIP`.
//
// ```jai
// text := "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) text, method = .DEFLATE }];
// bytes, error := encode_archive(items, .ZIP);
// defer free(bytes.data);
//
// archive, read_error := read_archive(bytes, .ZIP);
// defer free_archive(*archive);
// entry := archive.entries[0];
// assert(entry.method == .DEFLATE && entry.compressed_size < entry.size);
// ```
Archive_Method :: enum u16 {
    STORED = 0;
    DEFLATE = 8;
}

// Why an archive operation failed.
//
// ```jai
// _, error := read_archive(cast([] u8) "not an archive", .ZIP);
// defer free_archive(*error);
// assert(error.kind == .INVALID);
// ```
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

// The outcome of an operation; `kind` is `.NONE` on success. `entry` names the entry that failed,
// when there is one; release it with [free_archive].
//
// ```jai
// items := Archive_Item.[.{ name = "big.bin", data = cast([] u8) "12345" }];
// _, error := encode_archive(items, .ZIP, .{ max_entry = 4 });
// defer free_archive(*error);
// assert(error.kind == .LIMIT && error.entry == "big.bin");
// ```
Archive_Error :: struct {
    kind: Archive_Error_Kind;
    message: string; // static
    offset: s64; // byte position in the archive where it went wrong, when known
    entry: string; // owned; free_archive releases it
}

// Bounds applied while indexing, reading, writing and extracting, to contain hostile input.
//
// Exceeding one fails with `.LIMIT`. A negative bound, or a `max_ratio` of 0 or less, is not valid
// and also fails with `.LIMIT`. The defaults suit archives up to 1 GiB.
//
// ```jai
// items := Archive_Item.[.{ name = "a", data = cast([] u8) "x" }, .{ name = "b", data = cast([] u8) "y" }];
// bytes, _ := encode_archive(items, .ZIP);
// defer free(bytes.data);
//
// _, error := read_archive(bytes, .ZIP, .{ max_entries = 1 });
// defer free_archive(*error);
// assert(error.kind == .LIMIT);
// ```
Archive_Limits :: struct {
    max_archive: s64 = 1 << 30; // archive size in bytes
    max_metadata: s64 = 16 << 20; // total bytes of names, links and headers
    max_entries: s64 = 100000;
    max_entry: s64 = 256 << 20; // uncompressed size of one entry
    max_total: s64 = 1 << 30; // uncompressed size of all entries
    max_ratio: s64 = 1000; // largest uncompressed-to-stored size ratio of an entry
}

// A seekable byte source for [read_archive], such as a file read by offset.
//
// `read_at` fills `buffer` from `offset` and returns the count read and an empty string, or an error
// message; a short read is retried for the rest. The source must stay alive and unchanged while the
// [Archive] is used. Payloads read through a source are copied, so [read_entry] returns owned data.
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hi" }];
// bytes, _ := encode_archive(items, .TAR);
// defer free(bytes.data);
//
// read_at :: (data: *void, offset: s64, buffer: [] u8) -> s64, string {
//     whole := (cast(*[] u8) data).*;
//     memcpy(buffer.data, whole.data + offset, buffer.count);
//     return buffer.count, "";
// }
//
// archive, error := read_archive(Archive_Source.{ data = *bytes, size = bytes.count, read_at = read_at }, .TAR);
// defer free_archive(*archive);
// assert(error.kind == .NONE && archive.entries[0].name == "a.txt");
// ```
Archive_Source :: struct {
    data: *void; // passed back to `read_at`
    size: s64; // total bytes available
    read_at: (data: *void, offset: s64, buffer: [] u8) -> s64, string;
}

// One indexed archive entry. The strings are owned by the [Archive] that holds the entry.
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hello", mtime = 1700000000 }];
// bytes, _ := encode_archive(items, .TAR);
// defer free(bytes.data);
//
// archive, _ := read_archive(bytes, .TAR);
// defer free_archive(*archive);
// entry := archive.entries[0];
// assert(entry.kind == .FILE && entry.size == 5 && entry.mtime == 1700000000);
// ```
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

// An open archive from [read_archive]: the entry index plus what is needed to read payloads. Release
// it with [free_archive].
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "a" }, .{ name = "b.txt", data = cast([] u8) "b" }];
// bytes, _ := encode_archive(items, .ZIP);
// defer free(bytes.data);
//
// archive, _ := read_archive(bytes, .ZIP);
// defer free_archive(*archive);
// for archive.entries assert(it.size == 1);
// ```
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
// releases them; otherwise they borrow the archive's input. Calling [free_archive] is safe either way.
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hi" }];
// bytes, _ := encode_archive(items, .ZIP);
// defer free(bytes.data);
//
// archive, _ := read_archive(bytes, .ZIP);
// defer free_archive(*archive);
// data, _ := read_entry(*archive, 0);
// defer free_archive(*data);
// assert(!data.owned && cast(string) data.bytes == "hi"); // stored, so borrowed
// ```
Archive_Data :: struct {
    bytes: [] u8;
    owned: bool;
}

// An entry to write with [write_archive] or [encode_archive]. Nothing is copied: `name`, `data` and
// `link` must stay valid until the call returns.
//
// Names are UTF-8 without NUL bytes, with `/` between folders; a directory's name ends in `/`. Only
// a `.FILE` may have `data`.
//
// ```jai
// items := Archive_Item.[
//     .{ name = "src/", kind = .DIRECTORY },
//     .{ name = "src/main.jai", data = cast([] u8) "main :: () {}", method = .DEFLATE },
// ];
// bytes, error := encode_archive(items, .ZIP);
// defer free(bytes.data);
// assert(error.kind == .NONE);
// ```
Archive_Item :: struct {
    name: string;
    data: [] u8;
    kind: Archive_Kind;
    link: string; // the target of a symlink or hardlink (TAR only)
    mtime: s64; // seconds since the Unix epoch
    method: Archive_Method; // ZIP only
}

// Indexes the archive in `bytes` without copying stored payloads.
//
// Only the headers are read; payloads are checked when [read_entry] or [copy_entry] reads them.
// `bytes` must outlive the archive. A `.TAR_GZIP` archive is inflated into a buffer the archive owns,
// bounded by `limits`. On error the archive is empty and needs no freeing; free the error.
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hi" }];
// bytes, _ := encode_archive(items, .TAR);
// defer free(bytes.data);
//
// archive, error := read_archive(bytes, .TAR);
// defer free_archive(*archive);
// assert(error.kind == .NONE && archive.entries.count == 1);
// ```
read_archive :: (bytes: [] u8, format: Archive_Format, limits := Archive_Limits.{}) -> Archive, Archive_Error {
    a: Archive;
    a.bytes = bytes;
    a.source.size = bytes.count;
    a.format = format;
    a.limits = limits;

    return open_index(a);
}

// Indexes the archive in a seekable `source`, reading only headers. See [Archive_Source] for an
// example. A `.TAR_GZIP` source is read whole and inflated, since gzip cannot be read by offset.
read_archive :: (source: Archive_Source, format: Archive_Format, limits := Archive_Limits.{}) -> Archive, Archive_Error {
    a: Archive;
    a.source = source;
    a.format = format;
    a.limits = limits;

    return open_index(a);
}

// Returns the payload of entry `index`, checked against its size and, for ZIP, its CRC.
//
// A stored entry of an archive read from bytes borrows them; a deflated entry, or any entry read
// through an [Archive_Source], is a new buffer. Free the result with [free_archive] in both cases.
// An index out of range fails with `.INVALID`, a payload that does not match with `.CHECKSUM`. To
// avoid holding a large entry in memory, use [copy_entry].
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "compressed text text text", method = .DEFLATE }];
// bytes, _ := encode_archive(items, .ZIP);
// defer free(bytes.data);
//
// archive, _ := read_archive(bytes, .ZIP);
// defer free_archive(*archive);
// data, error := read_entry(*archive, 0);
// defer free_archive(*data);
// assert(error.kind == .NONE && data.owned && cast(string) data.bytes == "compressed text text text");
//
// _, out_of_range := read_entry(*archive, 5);
// defer free_archive(*out_of_range);
// assert(out_of_range.kind == .INVALID);
// ```
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

// Writes the payload of entry `index` to `writer`, checking its CRC as it goes.
//
// A stored entry is copied in 64 KiB pieces, so its size does not matter. A deflated entry is
// inflated whole first, as [read_entry] does. A CRC mismatch is found only after the bytes were
// written, so the writer must be ready to discard them on `.CHECKSUM`.
//
// ```jai
// items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hello" }];
// bytes, _ := encode_archive(items, .ZIP);
// defer free(bytes.data);
// archive, _ := read_archive(bytes, .ZIP);
// defer free_archive(*archive);
//
// count_bytes :: (data: *void, bytes: [] u8) -> bool {
//     (cast(*int) data).* += bytes.count;
//     return true;
// }
//
// total: int;
// error := copy_entry(*archive, 0, .{ data = *total, write_proc = count_bytes });
// assert(error.kind == .NONE && total == 5);
// ```
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

// Frees the entry index of `a` and, for `.TAR_GZIP`, the inflated bytes, then clears it. The input
// bytes or source passed to [read_archive] are not freed. An empty archive is fine to free.
//
// ```jai
// archive: Archive;
// free_archive(*archive);
// assert(archive.entries.count == 0);
// ```
free_archive :: (a: *Archive) {
    for a.entries {
        free(it.name);
        free(it.link);
    }

    array_free(a.entries);
    if a.owned.data free(a.owned.data);

    a.* = .{};
}

// Frees the bytes of `data` when it owns them, then clears it. See [Archive_Data].
free_archive :: (data: *Archive_Data) {
    if data.owned && data.bytes.data free(data.bytes.data);
    data.* = .{};
}

// Receives output bytes; returns false when it cannot take them all. `data` is the sink's own state.
Archive_Write_Proc :: #type (data: *void, bytes: [] u8) -> bool;

// A byte sink for [write_archive] and [copy_entry]. It owns nothing and works on every target, wasm
// included. To write to a file or socket, point `data` at it and forward to its own write.
//
// ```jai,only(linux, macos)
// Io :: #import "Extensions/Io";
// #import "File";
//
// to_file :: (data: *void, bytes: [] u8) -> bool {
//     _, error := Io.write(cast(*Io.Io_File) data, bytes);
//     return error == .NONE;
// }
//
// Io.io_run(() {
//     file, error := Io.open_file("/tmp/jacket-archive-writer.zip", .WRITE);
//     assert(error == .NONE);
//     items := Archive_Item.[.{ name = "a.txt", data = cast([] u8) "hi" }];
//     archive_error := write_archive(.{ data = *file, write_proc = to_file }, items, .ZIP);
//     Io.close(*file);
//     assert(archive_error.kind == .NONE);
// });
// file_delete("/tmp/jacket-archive-writer.zip");
// ```
Archive_Writer :: struct {
    data: *void;
    write_proc: Archive_Write_Proc;
}

// Frees the entry name of `error`, then clears it. Safe on an error with no name or on success. See
// [Archive_Error].
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
Utf8 :: #import "Extensions/Utf8";
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
