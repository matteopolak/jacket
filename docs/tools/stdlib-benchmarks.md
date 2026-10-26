# Stdlib benchmarks

## What it is

One program per performance-critical part of the standard library, in `benchmarks/runtime/stdlib-*.jai`. Each runs for a fraction of a second and prints a deterministic checksum, so two builds can be compared on retired instructions and checked for identical output. They are ordinary `generated-code runtime` workloads of the [benchmark harness](benchmarks.md).

| Program | What it exercises |
| --- | --- |
| `stdlib-table` | `Hash_Table` with integer keys: `table_set`, lookups that hit and miss |
| `stdlib-table-strings` | `Hash_Table` with string keys: set, hits, misses, `table_remove` |
| `stdlib-sort` | `quick_sort` of integers with a comparison procedure |
| `stdlib-sort-strings` | `quick_sort` of strings with `compare_strings`, `intro_sort` of integers |
| `stdlib-string-ops` | `String`: byte and substring search from both ends, `split`, `compare`, `equal`, `begins_with`, `join`, `to_upper_copy` / `to_lower_copy`, `replace`, `trim` |
| `stdlib-strings` | `print_to_builder` with integers and `formatFloat` |
| `stdlib-format-int` | integer formatting: `print_to_builder`, `formatInt` (base 16, minimum digits), `tprint` |
| `stdlib-utf8` | `Unicode`: `character_utf32_to_utf8`, `character_utf8_to_utf32`, `utf8_next_character` over one- to four-byte text |
| `stdlib-arrays` | `Basic`: `array_add` (one and several items), `array_find`, `array_unordered_remove_by_index` |
| `stdlib-alloc` | the default heap (a sliding window of mixed sizes), `talloc` with `reset_temporary_storage`, `Pool` and `Flat_Pool` through their allocator procedures |
| `stdlib-random` | `Random`: `random_get`, `random_get_zero_to_one`, `random_get_within_range` |
| `stdlib-math` | `Math`: `sqrt`, `sin`, `cos`, `atan2`, `exp`, `log`, `floor`, `abs` |
| `stdlib-bit-array` | `Bit_Array`: set, clear, toggle, iteration |
| `stdlib-paths` | `String` path helpers: `path_filename`, `path_strip_filename`, `path_extension`, `path_strip_extension` |
| `stdlib-scan` | `String` and `Basic` over long inputs: byte, any-of and substring search from both ends, `compare`, `array_find` on `s32` and `u8`, plus short-string searches that must not get slower |
| `stdlib-hash` | `Hash`: `fnv1a_hash`, `sdbm_hash`, `djb2_hash`, `get_hash` of integers and strings |

## How to run it

Retired instructions of every stdlib program, built by two compilers that each use their own standard library:

```sh
cargo build --release -p jaic-cli
PYTHONPATH=tools python3 tools/bench/main.py compare --baseline OTHER/jai --candidate target/release/jaic \
    --modes optimized --sizes '' --no-corpus --only=stdlib- --programs-only
```

Results go to `playground/bench/<timestamp>/summary.md` ("Built programs"); they are never committed. A program whose output differs between the two builds is listed under "Not timed": that is a behaviour difference in the library, not a benchmark problem. `--only=stdlib-utf8` narrows it to one program.

A quick look at one program with jaic alone:

```sh
target/release/jaic benchmarks/runtime/stdlib-utf8.jai -o -exe utf8 -output_path playground/
/usr/bin/time -l playground/utf8            # "instructions retired" on macOS
samply record playground/utf8               # where the time goes
```

`compare` builds through `tools/bench/phase_timing.jai` with `set_optimization(.OPTIMIZED)`, which leaves loop vectorization off; `jaic -o` turns it on. Compare numbers from one method only.

## What the library does to stay fast

These are the changes the programs above measure; keep them when editing the library.

- `Hash_Table`: `table_set` and `find_or_add` hash the key once and pass the hash to the lookup and the insert.
- `Hash`: `sdbm_hash` and `djb2_hash` fold four bytes per step (`value * M^4 + b0 * M^3 + ...`, with the powers wrapped to 32 bits). `fnv1a_hash` stays a plain loop: FNV-1a is serial per byte, and unrolling it stops the table lookup from inlining it, which costs more on short keys than it saves.
- `Unicode`: ASCII returns before any table lookup; two- to four-byte sequences are decoded and encoded without loops.
- Integer formatting: digits are written backwards into a stack buffer, two decimal digits per division (a `"00".."99"` table), shifts and masks for power-of-two bases, then appended once with the padding. `print_to_builder` appends literal text between `%` signs in one `append`, and `render_item` checks for integers before anything else.
- `String` searches: byte, any-of and substring searches switch to `Byte_Search` once the string is at least 64 bytes (substring: a 64-byte scalar probe first, vectors from 256 bytes), `compare` calls `memcmp` from 32 bytes, and the path helpers stay scalar (their needle is a constant the compiler unrolls). `stdlib-scan` runs about 5x fewer instructions than the scalar versions; `stdlib-paths` is unchanged.
- `array_find` without a custom `compare`, on 1-, 2-, 4- and 8-byte elements, tests 128 bytes per step with an OR-reduce LLVM vectorizes, and rescans only the block that hit (`internal/array-scan`). It is a plain loop, so it also runs on wasm.
- Measured and left alone: strict UTF-8 validation (the scalar loop is already fast on ASCII-heavy text) and the `%` scan in `print_to_builder` (`Basic` cannot import `Byte_Search`; its scan runs over short templates).
- `Random`: the float helpers scale the low 24 bits of one `random_get`.

Two programs outside the table are timed by themselves: `benchmarks/stdlib/byte-search.jai` (the 16-byte, 32-byte and picked versions of `Byte_Search`) and `benchmarks/stdlib/log-scan.jai` (a log scanner built on `Byte_Search`, about 0.9 GB/s on an Apple M-series core). On arm64 the 32-byte versions are two NEON registers wide; only x64 with AVX2 runs them as 256-bit code.

Float parsing and formatting (`formatFloat`, most of `stdlib-strings`) are tuned separately.

## How to change it

- A new hot path gets its own `benchmarks/runtime/stdlib-<area>.jai`; the harness picks it up by file name. Keep it between 0.2 and 1 s under `jaic -o`, end with one `print` of a checksum, and use only exports both compilers' standard libraries have, or `compare` cannot gate it.
- Build inputs with a small inline generator (an LCG) rather than `Random`, so the input does not depend on the library under test.
- Keep setup cheap next to the measured work: write keys straight into a buffer instead of formatting them.

## Dependencies

`tools/bench/` ([benchmarks and profiling](benchmarks.md)); `samply` or `/usr/bin/time -l` for profiling.
