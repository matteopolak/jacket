# `jacket bench`

## What it is

The benchmark runner for procedures with a `@bench` note. It builds them optimized, measures each in a process of its own, and prints a table or a JSON document; results can be saved as a baseline and later runs compared against it. Discovery, building, `-E` filters, notes such as `@only` and `--format` are those of [`jacket test`](jacket-test.md).

```jai
#import "Hash";
#import "Extensions/Testing";

hashing :: (b: *Bench) {
    set_bytes(b, 16);

    for 0..b.iterations - 1 {
        text := black_box("0123456789abcdef");
        black_box(fnv1a_hash(text.data, text.count));
    }
} @bench @samples(40)
```

```
$ jacket bench --path benchmarks/bench
benchmark      median    ± mad  throughput  allocs/iter
fnv1a         38.0 ns  0.19 ns   2.13 GB/s         0.00
hash_integer  1.07 ns  0.03 ns  934.58 M/s         0.00
join_words    66.4 ns  1.10 ns   60.21 M/s         1.00
```

Exit status: 0, 1 when a benchmark failed to run or regressed past `--max-regression`, 2 for a usage error, a missing baseline or a build that does not compile.

## How it works

**Where benchmarks are.** A `@bench` procedure takes nothing or a `*Bench` and sits at file level in a root's `benches/*.jai` or in a `tests/*.jai` file that contains `@bench`. `jacket bench` builds only those files (so unit tests and doc tests do not slow it); `jacket test` ignores `benches/`.

**Building.** The same Discovery build as `jacket test` (`build.rs`), with the plugin option `-optimize`: `Optimization_Type.OPTIMIZED` (LLVM O2), no bounds, null or cast checks, no stack traces. It is the plugin that does it because a plugin's build options replace the command line's, so `jaic -optimized` has no effect there. A flag after `--` that chooses the optimization (`-o`, `-od`, `-very_debug`, `-O0`..`-O3`) turns `-optimize` off. `--interp` and `-os wasm` are refused.

**Running.** Benchmarks run one at a time (a parallel run would measure the machine's load). Each is `PROGRAM --bench NAME`, and the program (`stdlib/Extensions/Testing/bench.jai`, `run_bench`) does:

1. **Warm-up** for about 100 ms with batches of 1, 2, 4... iterations. The last batch gives the time per iteration.
2. **Batch size**: iterations such that one sample takes about 10 ms.
3. **Samples**: N timed batches (default 30). Each sample is a time per iteration.
4. **Allocations**: one more batch whose `context.allocator` is wrapped by a counting allocator; allocations (including resizes) and bytes, divided by the iterations. It is separate so counting never slows the timed samples. Temporary storage is not counted.
5. A `bench` record (JSON) on standard output, in the framing of the [test protocol](jacket-test.md#the-test-program-protocol).

The benchmark body runs once per batch and loops `b.iterations` times; setup before the loop runs once per batch, so call `reset_timer(b)` after it to leave it out of the time and the allocation count.

**Statistics.** The median of the samples, the MAD (median absolute deviation: the spread, robust against a slow outlier), the minimum, maximum and the 5th and 95th percentiles. Throughput is computed from the median.

**Helpers** (all in `Extensions/Testing`):

| | |
| --- | --- |
| `black_box(value)` | returns `value`, hidden from the optimizer. Wrap the inputs inside the loop (or the optimizer computes a pure call once) and the results (or it drops the work). It costs a call through a function pointer, about 0.5 ns |
| `set_bytes(b, n)` | the iteration handles `n` bytes: MB/s in the report |
| `set_items(b, n)` | it handles `n` items: items/s |
| `reset_timer(b)` | restart the clock and the allocation count |
| `@samples(N)` | samples for this benchmark |
| `@min_time(MS)` | measure for at least MS in total: more samples when N samples would take less |

Other notes of `jacket test` work: `@only`, `@skip_on`, `@needs`, `@timeout(SECONDS)` (the default is `--timeout`, 120 s).

### Baselines

- `--save-baseline NAME` writes the JSON document below to `target/bench-baselines/NAME.json` (under the project folder).
- `--baseline NAME` compares each benchmark by name; the table gets a `change` column (positive is slower), and the JSON a `baseline` object. A benchmark the baseline lacks shows `new`.
- `--max-regression PCT` (needs `--baseline`) exits 1 when a median is more than PCT percent slower and the change is not noise. Noise: the difference is at most 3 times the larger MAD of the two runs. A change inside the noise shows as `(noise)` and never fails.

Both flags can be given at once: the comparison uses the old file, then it is overwritten.

### JSON document (`--format json`, and the baseline file)

One object on standard output; progress goes to standard error. `schema` changes when a field is removed or changes meaning (adding fields does not).

```json
{
  "schema": 1,
  "jacket": "0.29.0",
  "os": "macos",
  "cpu": "arm64",
  "results": [
    {
      "name": "fnv1a",
      "file": "/abs/benches/hash.jai",
      "line": 7,
      "samples": 30,
      "iterations": 263000,
      "median_ns": 37.9,
      "mad_ns": 0.1,
      "p5_ns": 37.8,
      "p95_ns": 38.6,
      "min_ns": 37.7,
      "max_ns": 39.0,
      "bytes_per_iteration": 81,
      "items_per_iteration": 0,
      "mb_per_s": 2137.0,
      "items_per_s": null,
      "allocs_per_iteration": 0.0,
      "alloc_bytes_per_iteration": 0.0,
      "baseline": { "median_ns": 37.9, "change_percent": 0.0, "noise": true, "regression": false }
    }
  ]
}
```

Times are nanoseconds per iteration. `baseline` is present only with `--baseline`; `regression` is `null` without `--max-regression`.

## How to change it

- The runner: `stdlib/Extensions/Testing/bench.jai` (`run_bench`, statistics, `black_box`). The bench table is written by `registry_source` in `Discovery/module.jai`; the notes are checked in `bench_note_problem`. Command line of the program: `runner.jai`.
- Reporting, baselines and exit status: `crates/jacket/src/test/bench.rs`. Flags: `BenchArgs` in `cli.rs`.
- Gotcha: `black_box` is only as opaque as `keeper` (`bench.jai`) makes it. A direct call to a procedure that does not read its argument lets LLVM delete the stores that fill it, and the benchmark measures nothing (it then reports about 1 ns). If a number looks too good, check how the optimizer sees the loop.
- Gotcha: `count_allocations` resets temporary storage, so do not keep temp allocations across it in the runner.

## Configuration

Flags: `--samples N`, `--warmup MS`, `--sample-ms MS` (the 30, 100 and 10 defaults), `--save-baseline`, `--baseline`, `--max-regression`; the selection and build options of `jacket test` (`FILTER`, `-E`, `--path`, `--format`, `--timeout`, `-os`/`-cpu`, flags after `--`). `[test] paths` in `jacket.toml` name the roots, as for tests.

## Dependencies

The `Extensions/Testing` module and its Discovery plugin; `jaic` with the LLVM backend. Not covered yet: the `tools/bench` runtime workloads (`benchmarks/runtime`) compare whole programs across two compiler builds and stay as they are; `benchmarks/bench/` holds `@bench` versions of the Hash module's functions as the first suite.
