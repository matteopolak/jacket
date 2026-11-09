// DEFLATE compression (RFC 1951) with its zlib (RFC 1950) and gzip (RFC 1952) wrappers.
// [deflate] compresses, [inflate] decompresses, and [crc32] and [adler32] are the checksums.
//
// ```jai
// #import "Extensions/Deflate";
//
// bytes := cast([] u8) "hello hello hello hello";
// packed := deflate(bytes, .GZIP);
// unpacked, ok, error := inflate(packed, .GZIP);
// assert(ok && cast(string) unpacked == "hello hello hello hello");
// ```
//
// Results are allocated with `context.allocator` and owned by the caller. Errors are static strings.

// The framing around the compressed blocks.
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
//
// ```jai
// #import "Extensions/Deflate";
//
// packed := deflate(cast([] u8) "abcabcabc", .ZLIB);
// output, ok, error := inflate(packed, .ZLIB);
// assert(ok);
// assert(cast(string) output == "abcabcabc");
//
// _, ok = inflate(cast([] u8) "not deflate data", .ZLIB);
// assert(!ok);
// ```
inflate :: (data: [] u8, container := Deflate_Container.ZLIB, max_output: s64 = DEFAULT_MAX_OUTPUT) -> output: [] u8, ok: bool, error: string {
    output, ok, error = inflate_member(data, container, max_output);
    return output, ok, error;
}

// Decompresses a complete protocol payload, rejecting trailing bytes. Gzip members
// are concatenated in order and share one aggregate decompressed-output limit.
// Every checksum must pass before any output is returned; release success with free.
//
// ```jai
// packed := deflate(cast([] u8) "hello", .GZIP);
// output, ok := inflate_complete(packed, .GZIP, 1024);
// assert(ok && cast(string) output == "hello");
// free(output.data);
// free(packed.data);
// ```
inflate_complete :: (data: [] u8, container := Deflate_Container.ZLIB, max_output: s64 = DEFAULT_MAX_OUTPUT) -> output: [] u8, ok: bool, error: string {
    if max_output < 0 return .[], false, "negative output limit";

    result: [..] u8;
    at: s64;

    while true {
        remaining: [] u8 = .{ data.count - at, data.data + at };
        member, valid, failure, consumed := inflate_member(remaining, container, max_output - result.count);

        if !valid || consumed <= 0 || consumed > remaining.count {
            array_reset(*result);
            if member.data free(member.data);
            return .[], false, ifx failure.count then failure else "invalid container length";
        }

        array_reserve(*result, result.count + member.count);
        if member.count memcpy(result.data + result.count, member.data, member.count);

        result.count += member.count;
        free(member.data);
        at += consumed;
        if at == data.count return .{ result.count, result.data }, true, "";

        if container != .GZIP {
            array_reset(*result);
            return .[], false, "trailing bytes after stream";
        }
    }
}

// Like [inflate], into a buffer the caller sized. Fails with "output is too small" when the
// stream expands past `output`; nothing is written beyond it.
//
// ```jai
// #import "Extensions/Deflate";
//
// packed := deflate(cast([] u8) "abcabcabc");
// buffer: [16] u8;
// written, ok := inflate_into(packed, buffer);
// assert(ok && written == 9);
// ```
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
//
// ```jai
// #import "Extensions/Deflate";
//
// text := cast([] u8) "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
// packed := deflate(text, .GZIP, 9);
// assert(packed.count < text.count);
// ```
deflate :: (data: [] u8, container := Deflate_Container.ZLIB, level: s32 = 6) -> [] u8 {
    return compress(data, container, level);
}

// CRC-32 (the checksum of gzip and PNG). To continue over more data, pass the previous result.
//
// ```jai
// #import "Extensions/Deflate";
//
// assert(crc32(cast([] u8) "123456789") == 0xCBF43926);
// ```
crc32 :: (data: [] u8, crc: u32 = 0) -> u32 {
    return crc32_update(data, crc);
}

// Adler-32 (the checksum of zlib). To continue over more data, pass the previous result.
//
// ```jai
// #import "Extensions/Deflate";
//
// assert(adler32(cast([] u8) "123456789") == 0x091E01DE);
// ```
adler32 :: (data: [] u8, adler: u32 = 1) -> u32 {
    return adler32_update(data, adler);
}

// Encodes a raw DEFLATE message ending at Z_SYNC_FLUSH, with the four-byte tail omitted.
// A borrowed dictionary seeds compression history; only its last 32 KiB are used.
// Example: packed := deflate_sync(cast([] u8) "hello"); free(packed.data).
deflate_sync :: (data: [] u8, level: s32 = 6, dictionary: [] u8 = .[]) -> [] u8 {
    return compress(data, .RAW, level, true, dictionary);
}

// Decodes a sync-flushed message with a borrowed history tail (at most 32 KiB).
// Output excludes the dictionary and must be freed. The output budget excludes history too.
// Example: plain, ok := inflate_sync(packed, .[], 1024); if ok free(plain.data).
inflate_sync :: (data: [] u8, dictionary: [] u8, max_output: s64 = DEFAULT_MAX_OUTPUT, window_bits: int = 15) -> output: [] u8, ok: bool, error: string {
    if window_bits < 8 || window_bits > 15 return .[], false, "invalid window bits";
    if dictionary.count > (1 << window_bits) || max_output < 0 || max_output > 0x7FFFFFFFFFFFFFFF - 32768 return .[], false, "invalid dictionary or output limit";

    input: [..] u8;

    defer array_free(input);
    array_reserve(*input, data.count + 9);
    if data.count memcpy(input.data, data.data, data.count);

    input.count = data.count;
    for u8.[0, 0, 255, 255, 1, 0, 0, 255, 255] array_add(*input, it);

    s := New(Inflater);

    defer free(s);
    s.growable = true;
    s.max_output = max_output + dictionary.count;
    s.window_limit = 1 << window_bits;
    s.cap = min(s.max_output, max(4096, dictionary.count));
    s.out = alloc(max(1, s.cap));
    if dictionary.count memcpy(s.out, dictionary.data, dictionary.count);

    s.o = dictionary.count;
    failure := inflate_container(s, input, .RAW);
    if failure.count || s.consumed != input.count {
        free(s.out);
        return .[], false, ifx failure.count then failure else "premature final DEFLATE block";
    }

    count := s.o - dictionary.count;
    for i: 0..count - 1 s.out[i] = s.out[i + dictionary.count];

    return .{ count, s.out }, true, "";
}

#load "tables.jai";
#load "checksum.jai";
#load "huffman.jai";
#load "inflate.jai";
#load "stream.jai";
#load "deflate.jai";

#scope_module

#import "Basic";

#scope_file

inflate_member :: (data: [] u8, container := Deflate_Container.ZLIB, max_output: s64 = DEFAULT_MAX_OUTPUT) -> output: [] u8, ok: bool, error: string, consumed: s64 {
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
        return .[], false, error, 0;
    }

    result: [] u8;
    result.data = s.out;
    result.count = s.o;

    return result, true, "", s.consumed;
}
