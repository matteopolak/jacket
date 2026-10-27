# Stdlib benchmarks

## What it is

The `@bench` suite for the standard library, in `benchmarks/bench/benches/`, run by [`jacket bench`](jacket-bench.md). One benchmark per performance-critical path, built optimized and measured in a process of its own, with a confidence interval, outliers and a comparison against a saved baseline.

| File | Benchmarks | What they exercise |
| --- | --- | --- |
| `tables.jai` | `table_int_insert`, `table_int_lookup`, `table_string_insert`, `table_string_lookup`, `table_string_remove` | `Hash_Table` with integer and string keys: `table_set`, lookups that hit and miss, `table_remove` |
| `sorting.jai` | `quick_sort_ints`, `quick_sort_strings`, `intro_sort_ints` | `quick_sort` with a comparison procedure and `compare_strings`, `intro_sort` |
| `string_ops.jai` | `string_ops_round`, `string_scan_long`, `string_scan_short`, `path_helpers` | `String`: searches from both ends, `split`, `compare`, `join`, case changes, `replace`, `trim`; long-input and short-string searches (the short ones must not get slower); the path helpers |
| `formatting.jai` | `format_item`, `format_integers` | `print_to_builder` with integers and `formatFloat`; `formatInt` (base 16, minimum digits) and `tprint` |
| `unicode.jai` | `utf8_encode`, `utf8_decode`, `utf8_next` | `Unicode` over one- to four-byte text: `character_utf32_to_utf8`, `character_utf8_to_utf32`, `utf8_next_character` |
| `arrays.jai` | `array_add_items`, `array_find_values`, `array_remove_unordered` | `Basic`: `array_add` (one and several items), `array_find`, `array_unordered_remove_by_index` |
| `allocators.jai` | `heap_window`, `temporary_storage`, `pool_allocations`, `flat_pool_allocations` | the default heap (a sliding window of mixed sizes), `talloc` with `reset_temporary_storage`, `Pool` and `Flat_Pool` through their allocator procedures |
| `numerics.jai` | `math_functions`, `random_values`, `bit_array_scatter`, `bit_array_iterate` | `Math` (`sqrt`, `sin`, `cos`, `atan2`, `exp`, `log`, `floor`, `abs`), `Random`, `Bit_Array` |
| `hash.jai` | `fnv1a`, `sdbm`, `djb2`, `hash_string`, `hash_integer`, `hash_mixed` | `Hash`: the string hashes and `get_hash` of integers and strings |
| `strings.jai` | `join_words`, `grow_array`, `empty_loop`, `empty_body` | `join` and array growth, and two loops that show what the report makes of almost nothing |

The cheap per-element ones (`math_functions`, `random_values`, `bit_array_scatter`, `path_helpers`, `string_scan_short`, `hash_mixed`) run one step of a loop per iteration, so they report nanoseconds per step. The others run a fixed amount of work per iteration (a sort of 3 000 000 integers, 4 000 000 table lookups) and report milliseconds, with `set_items` or `set_bytes` giving a rate. Such an iteration takes tens of milliseconds, so `jacket bench` samples them in its flat mode (whole iterations, at least 10).

## How to run it

```sh
jacket bench --path benchmarks/bench                      # all of them
jacket bench --path benchmarks/bench table_ utf8_         # those whose name contains one of these
jacket bench --path benchmarks/bench -E 'file(tables)'    # one file
```

To see what a library change did, save a baseline before it and compare after it:

```sh
jacket bench --path benchmarks/bench --save-baseline before
# edit stdlib/...
jacket bench --path benchmarks/bench --baseline before table_
```

The report says *regressed*, *improved*, *within noise* or *no change* for each benchmark, with a p-value; `--max-regression PCT` turns a regression into exit status 1. Baselines are written under `target/bench-baselines/`, never committed. Close other programs first: the numbers are as steady as the machine is idle.

Some benchmarks need a large input (a 2 000 000-entry table, a 1 500 000-character text). It is made once, on the first call, and kept in a global; `reset_timer(b)` after that keeps the first call from counting. Benchmarks that need a fresh container every iteration (inserts, sorts, removals) build it inside the loop and free it, so the allocation and the free are part of the measurement, as they were in the programs these replace.

## What the library does to stay fast

These are the changes the benchmarks above measure; keep them when editing the library.

- `Hash_Table`: `table_set` and `find_or_add` hash the key once and pass the hash to the lookup and the insert.
- `Hash`: `sdbm_hash` and `djb2_hash` fold four bytes per step (`value * M^4 + b0 * M^3 + ...`, with the powers wrapped to 32 bits). `fnv1a_hash` stays a plain loop: FNV-1a is serial per byte, and unrolling it stops the table lookup from inlining it, which costs more on short keys than it saves.
- `Unicode`: ASCII returns before any table lookup; two- to four-byte sequences are decoded and encoded without loops.
- Integer formatting: digits are written backwards into a stack buffer, two decimal digits per division (a `"00".."99"` table), shifts and masks for power-of-two bases, then appended once with the padding. `print_to_builder` appends literal text between `%` signs in one `append`, and `render_item` checks for integers before anything else.
- `String` searches: byte, any-of and substring searches switch to `Byte_Search` once the string is at least 64 bytes (substring: a 64-byte scalar probe first, vectors from 256 bytes), `compare` calls `memcmp` from 32 bytes, and the path helpers stay scalar (their needle is a constant the compiler unrolls). `string_scan_long` is about 5x faster than with the scalar versions; `path_helpers` is unchanged.
- `array_find` without a custom `compare`, on 1-, 2-, 4- and 8-byte elements, tests 128 bytes per step with an OR-reduce LLVM vectorizes, and rescans only the block that hit (`internal/array-scan`). It is a plain loop, so it also runs on wasm.
- Measured and left alone: strict UTF-8 validation (the scalar loop is already fast on ASCII-heavy text) and the `%` scan in `print_to_builder` (`Basic` cannot import `Byte_Search`; its scan runs over short templates).
- `Random`: the float helpers scale the low 24 bits of one `random_get`.

Two programs outside the table are timed by themselves: `benchmarks/stdlib/byte-search.jai` (the 16-byte, 32-byte and picked versions of `Byte_Search`) and `benchmarks/stdlib/log-scan.jai` (a log scanner built on `Byte_Search`, about 0.9 GB/s on an Apple M-series core). On arm64 the 32-byte versions are two NEON registers wide; only x64 with AVX2 runs them as 256-bit code.

Float parsing and formatting (`formatFloat`, most of `format_item`) are tuned separately.

## How to change it

- A new hot path gets a `@bench` procedure in the file of its area under `benchmarks/bench/benches/` (or a new file there). Follow the rules in [`jacket bench`](jacket-bench.md): wrap inputs inside the loop and results in `black_box`, or the optimizer computes a pure call once and the loop measures nothing (about 0.3 ns). Procedure names are shared by every file of the suite, so keep them distinct.
- Build inputs with a small inline generator (an LCG) rather than `Random`, so the input does not depend on the library under test.
- Keep setup out of the loop: build large inputs once in a global on the first call, or call `reset_timer(b)` after setting up.
- Aim for 1 us or more per iteration, or a per-step body that is a few nanoseconds; an iteration over about 0.3 s makes a run slow (`@samples(10)` keeps it short).

## Configuration

The flags of [`jacket bench`](jacket-bench.md); the notes `@samples(N)` and `@min_time(MS)` on a benchmark. Lint: `benchmarks/bench/**` is exempt from `magic_number` in `jacket.toml`, since sizes and seeds are the workload.

## Dependencies

`jacket bench` and the `Extensions/Testing` module ([`jacket bench`](jacket-bench.md)). The compiler harness in `tools/bench/` ([benchmarks and profiling](benchmarks.md)) does not run these; it times whole compiler processes and the programs in `benchmarks/runtime/`.
