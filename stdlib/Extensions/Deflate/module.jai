// Deflate: the DEFLATE compressed format (RFC 1951) with its zlib (RFC 1950) and gzip (RFC 1952)
// wrappers, in plain Jai. A jaic extension, not official Jai. See docs/extensions/deflate.md.
//
//     #import "Extensions/Deflate";
//
//     packed := deflate(bytes, .GZIP);
//     unpacked, ok, error := inflate(packed, .GZIP);
//
// Results are allocated with context.allocator and owned by the caller. Errors are static strings.

Deflate_Container :: enum u8 {
    RAW; // bare DEFLATE blocks
    ZLIB; // 2-byte header, Adler-32 trailer (PNG's IDAT stream)
    GZIP; // gzip member: header, CRC-32 and size trailer
}

// A gzip member is at least this long: 10 bytes of header and 8 of trailer.
GZIP_MIN_SIZE :: 18;

// DEFLATE cannot expand a byte past 1032 bytes of output.
MAX_RATIO :: 1032;

// inflate refuses to produce more than this many bytes unless told otherwise.
DEFAULT_MAX_OUTPUT :: 1 << 30;

// Decompresses `data` into a new buffer. `max_output` bounds the result (a compressed bomb fails
// instead of exhausting memory). A GZIP stream is a single member; bytes after the stream are
// ignored. On failure the output is empty and `error` says why.
inflate :: (data: [] u8, container := Deflate_Container.ZLIB, max_output: s64 = DEFAULT_MAX_OUTPUT) -> output: [] u8, ok: bool, error: string {
    s := New(Inflater);

    defer free(s);

    s.growable = true;
    s.max_output = max_output;

    hint := data.count * 4;

    // A gzip trailer states the size; believe it when a stream this small could produce it.
    if container == .GZIP && data.count >= GZIP_MIN_SIZE {
        size := cast(s64) le32(data.data + data.count - 4);
        if size <= data.count * MAX_RATIO hint = size;
    }

    hint = max(min(max(hint, 4096), max_output), 0);

    s.cap = hint;
    s.out = alloc(max(hint, 1));

    error := inflate_container(s, data, container);

    if error.count {
        free(s.out);
        return .[], false, error;
    }

    result: [] u8;
    result.data = s.out;
    result.count = s.o;

    return result, true, "";
}

// Like `inflate`, into a buffer the caller sized. Fails with "output is too small" when the
// stream expands past `output`; nothing is written beyond it.
inflate_into :: (data: [] u8, output: [] u8, container := Deflate_Container.ZLIB) -> written: s64, ok: bool, error: string {
    s := New(Inflater);

    defer free(s);

    s.out = output.data;
    s.cap = output.count;

    error := inflate_container(s, data, container);
    if error.count return 0, false, error;

    return s.o, true, "";
}

// Compresses `data`. `level` runs from 0 (stored blocks only) to 9 (smallest); 6 is the usual
// balance. Out-of-range levels are clamped.
deflate :: (data: [] u8, container := Deflate_Container.ZLIB, level: s32 = 6) -> [] u8 {
    return compress(data, container, level);
}

// CRC-32 (the checksum of gzip and PNG). To continue over more data, pass the previous result.
crc32 :: (data: [] u8, crc: u32 = 0) -> u32 {
    return crc32_update(data, crc);
}

adler32 :: (data: [] u8, adler: u32 = 1) -> u32 {
    return adler32_update(data, adler);
}

#load "tables.jai";
#load "checksum.jai";
#load "huffman.jai";
#load "inflate.jai";
#load "stream.jai";
#load "deflate.jai";

#scope_module

#import "Basic";
