# Stdlib benchmarks

The `@bench` suite for the standard library, in `benchmarks/bench/benches/`, run by [`jacket bench`](jacket-bench.md). One benchmark per performance-critical path.

| File | Exercises |
| --- | --- |
| `tables.jai` | `Hash_Table` with integer and string keys: set, hit and miss lookups, remove |
| `sorting.jai` | `quick_sort` and `intro_sort` |
| `string_ops.jai` | `String` searches, `split`, `join`, `replace`, `trim`, path helpers |
| `formatting.jai` | `print_to_builder`, `formatInt`, `tprint` |
| `unicode.jai` | UTF-8 encode, decode and step |
| `arrays.jai` | `array_add`, `array_find`, unordered remove |
| `allocators.jai` | the default heap, temporary storage, `Pool`, `Flat_Pool` |
| `numerics.jai` | `Math`, `Random`, `Bit_Array` |
| `hash.jai` | the string hashes and `get_hash` |
| `strings.jai` | `join`, array growth, and two empty loops that show what the report makes of nothing |

Cheap per-element benchmarks run one loop step per iteration and report nanoseconds. The others do a fixed amount of work per iteration (sorting a few million integers) and report milliseconds; `jacket bench` samples those in flat mode.

```sh
jacket bench --path benchmarks/bench                      # all
jacket bench --path benchmarks/bench table_ utf8_         # names containing one of these
jacket bench --path benchmarks/bench -E 'file(tables)'    # one file
```

To see what a library change did, save a baseline first:

```sh
jacket bench --path benchmarks/bench --save-baseline before
# edit stdlib/...
jacket bench --path benchmarks/bench --baseline before table_
```

Baselines go to `target/bench-baselines/` and are never committed. Close other programs first.

Byte search and log scanning run with `jacket bench --path benchmarks/stdlib`; Regex runs with `jacket bench --path benchmarks/regex/jai`. Extension benchmarks live in each module's `benches/` folder and run with `jacket bench --path stdlib/Extensions/<Module>`. Each input and mode is a distinct `@bench`, including reference-library cases where present.

Prepare external data with `PYTHONPATH=tools python3 tools/portable/build_portable.py data`. Missing inputs fail with the fetch command. JSON inputs can also be fetched with `PYTHONPATH=tools python3 tools/dev/json_bench_data.py`. Generated data and scratch files use `target/`, relocated by `JACKET_OUTPUT_DIR`.

For a short sequential smoke check, use `--samples 2 --warmup 1 --measure-ms 1 --resamples 100 -j 1`. These results check that cases run; they are not performance comparisons. TLS, WebSocket and Postgres cases start their own local peers before timing. HTTP's server, client RSS and child-process RSS experiments remain standalone programs: they need a peer that persists across samples or resident-memory/stack-residency metrics the runner does not provide.

## How to change it

- Add a `@bench` procedure to the file of its area (or a new file). Wrap inputs inside the loop and results in `black_box`, or the optimizer computes a pure call once and the loop measures nothing. Procedure names are shared across the suite, so keep them distinct.
- Build inputs with a small inline generator (an LCG), not `Random`, so they don't depend on the library under test.
- Keep setup out of the loop: build large inputs once in a global, or call `reset_timer(b)` after setting up.
- Aim for 1 us or more per iteration, or a few nanoseconds per step. Use `@samples(10)` for iterations over about 0.3 s.

## Configuration

The flags of `jacket bench`, plus `@samples(N)` and `@min_time(MS)` on a benchmark. `benchmarks/bench/**` is exempt from `magic_number` in `jacket.toml`.

## Dependencies

`jacket bench` and `Extensions/Testing`. The compiler harness in `tools/bench/` ([benchmarks](benchmarks.md)) does not run these.
