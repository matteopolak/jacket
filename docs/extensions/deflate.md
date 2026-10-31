# Deflate

DEFLATE compression and decompression (RFC 1951) with zlib and gzip framing, plus CRC-32 and Adler-32, in plain Jai: no `#foreign`, so it runs natively, under `jaic -interp`, in `-os wasm` and in the playground. `Extensions/Image` uses it for PNG.

```jai
#import "Extensions/Deflate";

packed := deflate(bytes, .ZLIB, 6);                       // level 0 (stored) to 9
unpacked, ok, error := inflate(packed, .ZLIB);            // max_output defaults to 1 GiB
written, ok2, error2 := inflate_into(packed, buffer);     // no allocation
crc := crc32(bytes);                                      // crc32(more, crc) continues
```

Results use `context.allocator` and belong to the caller. Errors are static strings (`bad zlib header`, `output exceeds the limit`, ...).

`.RAW` is bare blocks, `.ZLIB` adds a header and Adler-32 (PNG's IDAT), `.GZIP` is one member with CRC-32 and size trailer. Decoding checks headers and trailers and ignores bytes after the stream. Concatenated gzip members and zlib preset dictionaries are not supported.

## How it works

- Inflate (`inflate.jai`, `stream.jai`): a 64-bit bit buffer, one-level decode tables with sub-tables for long codes, and a `#no_abc` fast loop that runs while there are 8 input bytes and 274 bytes of output room. A fully checked loop takes over near either end, so truncated input and exact-size buffers are safe. Over-subscribed and incomplete codes are rejected.
- Deflate (`deflate.jai`, `huffman.jai`): hash chains over a 32 KiB window, lazy matching from level 4, per-level tuning in `settings_for`. Each block is written as the smallest of stored, fixed or dynamic Huffman. Inputs over 1 GiB are compressed in independent segments.
- Checksums (`checksum.jai`): CRC-32 by slicing-by-8 over a compile-time table; Adler-32 in 32-byte groups.

Inflate runs at roughly 2.5x stb's speed on text, with level 6 compressing text to 0.20 of its size (stb: 0.30). Run `bench/bench.jai` for numbers:

```sh
jaic -O2 stdlib/Extensions/Deflate/bench/bench.jai -exe /tmp/deflate-bench && /tmp/deflate-bench
```

## How to change it

- Encoder tuning: `settings_for`, `BLOCK_TOKENS`, `WINDOW`. Judge ratio and speed on a real corpus, not only generated inputs.
- New container or option: `stream.jai` and the head/tail of `compress`. The public API lives only in `module.jai` (fuzz discovery reads it).
- Decoder tables: an entry is `kind | value << 16 | codelen << 5 | total_bits`. Keep every index the `#no_abc` loop uses inside the table, and `FAST_OUTPUT_ROOM` at least the longest write of `copy_match`.
- Gotcha: `cast(T) x << n` shifts in `x`'s type before casting; parenthesize.
- Tests: `tests/roundtrip.jai`, `vectors.jai`, `malformed.jai`, `limits.jai`, and `stb.jai` (native only). `#no_abc` code doesn't trap out-of-bounds reads in the interpreter, so after touching the fast loop also run natively with `-sanitize address`. Fuzz: `PYTHONPATH=tools python3 tools/fuzz/stdlib_fuzz.py Extensions/Deflate --count 500`.
