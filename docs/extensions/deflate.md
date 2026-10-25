# Deflate (`Extensions/Deflate`)

## What it is

DEFLATE compression and decompression (RFC 1951) with the zlib (RFC 1950) and gzip (RFC 1952) wrappers, plus CRC-32 and Adler-32, in plain Jai: no `#foreign`, so it runs natively, under `jaic -interp`, in `-os wasm` programs and in the playground. `Extensions/Image` uses it for PNG; gzip and zip readers build on the same procedures.

```jai
#import "Extensions/Deflate";

packed := deflate(bytes, .ZLIB, 6);                       // level 0 (stored) to 9; defaults: .ZLIB, 6
unpacked, ok, error := inflate(packed, .ZLIB);            // max_output defaults to 1 GiB
written, ok2, error2 := inflate_into(packed, buffer);     // no allocation; "output is too small" if it does not fit
crc := crc32(bytes);                                      // crc32(more, crc) continues; same for adler32(more, adler)
```

Results use `context.allocator` and belong to the caller (`free(result.data)`). Errors are static strings: `unexpected end of data`, `bad zlib header`, `adler32 checksum mismatch`, `invalid block type`, `distance is before the start of the output`, `output exceeds the limit`, and so on (grep `inflate.jai`/`stream.jai`).

`.RAW` is bare blocks, `.ZLIB` has the 2-byte header and Adler-32 trailer (PNG's IDAT), `.GZIP` is one member with header and CRC-32/size trailer. Decoding checks the headers and trailers; bytes after the stream are ignored. Gzip support is a single member (no concatenated members; read the next one from the end of the previous yourself if you need it); `FEXTRA`, `FNAME`, `FCOMMENT` and `FHCRC` are parsed (and `FHCRC` verified). zlib preset dictionaries (`FDICT`) are rejected. The encoder writes `MTIME` 0 and OS 255 (unknown).

## How it works

- **Inflate** (`inflate.jai`, `stream.jai`). A 64-bit bit buffer refilled eight bytes at a time without branching. Literal/length and distance codes decode through one-level tables of `u32` entries (10 root bits for literals/lengths, 8 for distances) with sub-tables for longer codes; an entry carries the bits to consume (code plus extra bits), the kind and the value, so a length is one lookup plus a mask. The fast loop runs while there are 8 input bytes left and 274 bytes of output room (the longest match plus word-copy slack): matches are copied 16 bytes at a time when the distance allows, with a period trick for distances under 8, and the loop is `#no_abc`. Near the end of either buffer a fully checked loop takes over, one symbol at a time, so truncated input and exact-size output buffers are safe. `inflate` grows its buffer by `realloc` (starting from 4x the input, or the gzip ISIZE when believable) and refuses past `max_output`; `inflate_into` has no growth and reports `output is too small`.
- **Code validity.** Over-subscribed and incomplete codes are rejected, except a lone code of length 1 (a block that uses one distance, or none, may say so) and an empty distance code. The code length code must be complete. `HLIT` over 286 and `HDIST` over 30 are errors.
- **Deflate** (`deflate.jai`, `huffman.jai`). Hash chains over 3 (levels 6 to 9) or 4 bytes (levels 1 to 5) with a 32 KiB window, lazy matching from level 4, and per-level chain depth, "nice length" cut-off and lazy thresholds in `settings_for`. Tokens collect up to 65536 per block; each block is written as the smallest of stored, fixed Huffman and dynamic Huffman (computed from the exact symbol counts). Dynamic codes are length-limited to 15 bits (Huffman, then a Kraft-sum repair when too deep) and the code length alphabet to 7. Level 0 writes stored blocks. Inputs over 1 GiB are compressed in independent 1 GiB segments (chain positions are 32-bit).
- **Checksums** (`checksum.jai`). CRC-32 by slicing-by-8 over a table built at compile time (`#run`); Adler-32 in 32-byte groups without a serial dependency, one modulo per 5552 bytes.

Speed on an arm64 Mac, `-O2`, MB/s of uncompressed data (`bench/bench.jai`; stb is stb_image's `stbi_zlib_decode_malloc` and stb_image_write's `stbi_zlib_compress` at quality 8; ratio is compressed over input):

| Input | inflate | stb | deflate L1 | L6 | L9 | stb compress | ratio L6 / stb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| text (6 MB) | 1575 | 614 | 262 | 73 | 14 | 70 | 0.20 / 0.30 |
| random bytes | 11858 | 16173 | 104 | 101 | 103 | 36 | 1.00 / 1.00 |
| image rows | 735 | 400 | 191 | 27 | 11 | 41 | 0.44 / 0.68 |
| repetitive (16 MB) | 5941 | 5670 | 1018 | 991 | 361 | 474 | 0.013 / 0.024 |

Inflate verifies the checksum and stb does not; on stored data that is the whole difference (Adler-32 runs at about 15 GB/s, CRC-32 at about 2.7 GB/s). On real data (8 MB of this repository's `.jai` sources, 8 MB of the `jaic` binary) level 6 gives 0.192 and 0.409 at 150 and 91 MB/s.

## How to change it

- **Tune the encoder:** `settings_for` (chain depth, `nice`, `lazy`, `good`, `min_match` per level). Check ratio and speed with `bench/bench.jai` and a real corpus, not only the generated inputs; a 3-byte minimum helps text and costs time on noisy data. Block size is `BLOCK_TOKENS`; the window is `WINDOW` (a distance of exactly 32768 is legal and used).
- **A new container or option** goes in `stream.jai` (`inflate_container`, `parse_gzip_header`) and the head/tail of `compress` in `deflate.jai`; the public API lives only in `module.jai` (the fuzz discovery reads that file).
- **Decoder tables:** an entry is `kind | value << 16 | codelen << 5 | total_bits` (see the comment in `inflate.jai`). `build_table` guarantees every index the `#no_abc` fast loop uses is inside the table; keep that invariant if you change widths (`LITERAL_ROOT`, `DISTANCE_ROOT`, table sizes). The output margin `FAST_OUTPUT_ROOM` must cover the longest write of `copy_match`.
- **Gotcha:** `cast(T) x << n` shifts in the type of `x` before casting. Put the cast in parentheses or use a typed local; the code does so throughout.
- Tests: `tests/roundtrip.jai`, `vectors.jai`, `malformed.jai`, `limits.jai` (all modes) and `stb.jai` (native only). Malformed-input tests count allocations with a tracking allocator. The fuzz hooks are `tools/fuzz/properties/Deflate.jai` (`python3 tools/fuzz/stdlib_fuzz.py Extensions/Deflate --count 500`). `#no_abc` code does not trap out-of-bounds reads in the interpreter: after touching the fast loop, also run the tests natively with `-sanitize address`.

## Configuration

`DEFAULT_MAX_OUTPUT` (1 GiB; `inflate`'s `max_output`), the `level` argument (0 to 9, clamped), `BLOCK_TOKENS`, `WINDOW`, `SEGMENT`, and the table sizes in `inflate.jai`. No environment variables.

Benchmark (native only, needs the stb libraries in `JAIC_NATIVE_LIBS`):

```sh
jaic -O2 stdlib/Extensions/Deflate/bench/bench.jai -exe /tmp/deflate-bench && /tmp/deflate-bench
```

## Dependencies

`Basic` only; the bench and `tests/stb.jai` also use `stb_image`, `stb_image_write` (and `Random` in the bench). The algorithms are the ones in RFC 1950, 1951 and 1952; the code is written for this repo.
