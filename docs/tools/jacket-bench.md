# `jacket bench`

The benchmark runner for procedures with a `@bench` note. It builds them optimized, measures each in its own process, and reports in the manner of criterion: time per iteration with a bootstrap confidence interval, the change against a saved baseline with a p-value and a verdict, and outliers. Discovery, `-E` filters, notes such as `@only`, and `--format` are those of [`jacket test`](jacket-test.md).

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
$ jacket bench --path benchmarks/bench --baseline main
fnv1a                   time:   [41.24 41.30 41.37 ns]
                        thrpt:  [1.939 1.943 1.946 GB/s]
                        change: [-0.41% -0.19% +0.05%] (p = 0.12 > 0.05)
                        No change in performance detected.
```

Exit status: 0; 1 when a benchmark failed or regressed past `--max-regression`; 2 for a usage error, a missing baseline or a build that does not compile.

## How it works

- **Where.** A `@bench` procedure (taking nothing or a `*Bench`) sits at file level in a root's `benches/*.jai`, or in a `tests/*.jai` file that contains `@bench`. `jacket test` ignores `benches/`.
- **Build.** The Discovery build as for tests, with `-o -no_debug_info`: LLVM O2, no checks, no debug info. A flag after `--` that picks an optimization level overrides it. `--interp` and `-os wasm` are refused.
- **Run.** One benchmark at a time, each as `PROGRAM --bench NAME` (`stdlib/Extensions/Testing/bench.jai`, `run_bench`):
  1. Pin to the current core (Linux only).
  2. Warm up until the last five batches agree within 2%.
  3. Take `--samples` linear samples (sample k runs `k * step` iterations) totalling about `--measure-ms`. Very slow benchmarks get flat samples of whole iterations.
  4. Retake the samples (up to three times) if they do not lie on a line (R² under 0.99).
  5. Count allocations in one extra batch.
  6. Print a `bench` record (JSON) with the raw samples.
- **Statistics** (`crates/jacket/src/test/bench_stats.rs`, seeded, so one record gives one report): time is the least-squares slope through the origin; intervals are 95% bootstrap intervals; outliers use Tukey's fences. A baseline comparison is *regressed* or *improved* when p is below `--significance` and the whole interval lies beyond `--noise-threshold`, *within noise* when only p passes, and *no change* otherwise.
- Setup before the loop runs once per sample; call `reset_timer(b)` after it to leave it out.

Helpers in `Extensions/Testing`:

| | |
| --- | --- |
| `black_box(value)` | returns `value`, hidden from the optimizer. Wrap inputs inside the loop and the results too, or the work is hoisted or dropped |
| `set_bytes(b, n)` / `set_items(b, n)` | report MB/s or items/s |
| `reset_timer(b)` | restart the clock and the allocation count |
| `@samples(N)`, `@min_time(MS)` | per-benchmark samples and sampling time |

`@only`, `@skip_on`, `@needs` and `@timeout(SECONDS)` work as in `jacket test`.

### Baselines

- `--save-baseline NAME` writes `target/bench-baselines/NAME.json`, with the raw samples.
- `--baseline NAME` compares by benchmark name and adds `change:` lines (a missing one shows `new`). A baseline with another `schema` is refused; save it again.
- `--max-regression PCT` (needs `--baseline`) exits 1 when a benchmark regressed by more than PCT percent.

### JSON (`--format json`, and the baseline file)

One object on standard output; progress goes to standard error. `schema` changes when a field is removed or changes meaning. Each entry in `results` has `name`, `file`, `line`, `mode` (`linear` or `flat`), `time_ns`, `mean_ns`, `median_ns` (each `{low, estimate, high}`), `r_squared`, `outliers`, `mb_per_s`, `items_per_s`, `allocs_per_iteration`, the raw `sample_iterations`/`sample_ns`, and with `--baseline` a `baseline` object (`change`, `change_percent`, `p_value`, `verdict`). Times are ns per iteration; `change` is a fraction.

## How to change it

- Benchmark process: `stdlib/Extensions/Testing/bench.jai`. The bench table is written by `registry_source` in `Discovery/module.jai`; notes are checked in `bench_note_problem`.
- Statistics: `bench_stats.rs`. Report, baselines and exit status: `bench.rs`. Flags: `BenchArgs` in `cli.rs`.
- Gotcha: a benchmark is only as honest as its `black_box` calls. If a number looks too good (about 0.3 ns), the optimizer probably deleted the loop.
- Gotcha: `count_allocations` resets temporary storage.

## Configuration

| Flag | Default | |
| --- | --- | --- |
| `--samples N` | 50 | samples per benchmark |
| `--warmup MS` | 100 | least warm-up |
| `--measure-ms MS` | 1000 | time spent sampling |
| `--resamples N` | 100000 | bootstrap resamples |
| `--noise-threshold PCT` | 2 | smaller changes are noise |
| `--significance P` | 0.05 | p-value cutoff |

Plus the selection and build options of `jacket test`. Close other programs for steadier numbers.

## Dependencies

`Extensions/Testing` and its Discovery plugin; `jaic` with the LLVM backend. [`tools/bench`](benchmarks.md) is the maintainer harness for the compiler and tools; the standard library's suite is in [stdlib benchmarks](stdlib-benchmarks.md).
