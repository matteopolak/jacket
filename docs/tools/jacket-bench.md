# `jacket bench`

## What it is

The benchmark runner for procedures with a `@bench` note. It builds them optimized, measures each in a process of its own, and reports in the manner of criterion: the time per iteration with a bootstrap confidence interval, the change against a saved baseline with a p-value and a verdict, and the outliers. Discovery, building, `-E` filters, notes such as `@only` and `--format` are those of [`jacket test`](jacket-test.md).

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
                        R² 0.9999   median [41.26 41.31 41.40 ns]   allocs/iter 0.00
                        change: [-0.41% -0.19% +0.05%] (p = 0.12 > 0.05)
                        No change in performance detected.
Found 3 outliers among 50 measurements (6.00%)
  2 (4.00%) low mild
  1 (2.00%) high severe
```

Exit status: 0, 1 when a benchmark failed to run or regressed past `--max-regression`, 2 for a usage error, a missing baseline or a build that does not compile.

## How it works

**Where benchmarks are.** A `@bench` procedure takes nothing or a `*Bench` and sits at file level in a root's `benches/*.jai` or in a `tests/*.jai` file that contains `@bench`. `jacket bench` builds only those files (so unit tests and doc tests do not slow it); `jacket test` ignores `benches/`.

**Building.** The same Discovery build as `jacket test` (`build.rs`), with `-o -no_debug_info` passed to jaic: LLVM O2, no bounds, null or cast checks, no stack traces, no debug information. A flag after `--` that chooses the optimization (`-o`, `-od`, `-very_debug`, `-O0`..`-O3`) leaves them out. `--interp` and `-os wasm` are refused.

**Running.** Benchmarks run one at a time (a parallel run would measure the machine's load). Each is `PROGRAM --bench NAME`, and the program (`stdlib/Extensions/Testing/bench.jai`, `run_bench`) does:

1. **Pinning.** On Linux the process pins itself to the core it is on (`sched_setaffinity`), so the scheduler does not move it mid-run. macOS has no way to pin a thread to a core; there the number is as steady as the machine is idle.
2. **Warm-up.** Batches that double find the order of magnitude, then batches of about 10 ms go on until at least `--warmup` has passed and the last five agree within 2%. A benchmark that never settles stops after ten times that.
3. **Sampling.** Linear, as criterion does it: sample k runs `k * step` iterations for k = 1..N (N is 50), with `step` chosen so the samples add up to `--measure-ms` (1 s). A benchmark so slow that one step would not fit gets equal samples of whole iterations instead (`"mode": "flat"`), and at least 10 of them. Each sample is two clock reads around one call of the benchmark body; the body is a loop over `b.iterations`, so nothing else runs per iteration.
4. **Retry.** If the samples do not lie on a line (R² under 0.99, as when another process takes the core for a while), they are taken again, up to three times, and the cleanest take is kept (`attempts` in the report).
5. **Allocations.** One more batch whose `context.allocator` is wrapped by a counting allocator; allocations (including resizes) and bytes, divided by the iterations. Temporary storage is not counted.
6. A `bench` record (JSON) with the raw (iterations, nanoseconds) pairs on standard output, in the framing of the [test protocol](jacket-test.md#the-test-program-protocol).

The benchmark body runs once per sample and loops `b.iterations` times; setup before the loop runs once per sample, so call `reset_timer(b)` after it to leave it out of the time and the allocation count.

**Statistics** (`bench_stats.rs`, in jacket, not in the benchmark process; seeded, so one record always gives one report):

- **Time** is the slope of the least-squares line through (iterations, nanoseconds), forced through the origin as criterion does. What a sample costs besides its iterations (two clock reads and a call, about 0.1 us) is under 0.02% of the shortest sample, so it needs no intercept, which would only add noise; the free fit's intercept is in the JSON as `overhead_ns` for a check. `R²` says how well the samples lie on the line. A flat run has no line: the estimate is total time over total iterations.
- **Intervals** are 95% bootstrap intervals (`--resamples`, 100000) for the time, the mean and the median of the per-iteration times.
- **Outliers** are the per-iteration times beyond Tukey's fences, 1.5 (mild) and 3 (severe) times the interquartile range from the quartiles.
- **Comparison with a baseline:** the change is the ratio of the two time estimates minus one, with a bootstrap interval. The p-value is the share of Welch t statistics, computed from both runs pooled, as extreme as the observed one. The verdict is *regressed* or *improved* when p is below `--significance` (0.05) and the whole interval lies beyond the noise threshold (`--noise-threshold`, 2%), *within noise* when p is below it but the change is small, and *no change* otherwise.
- **The loop itself.** `black_box` costs no instruction, so an empty loop that uses `it` measures one cycle of loop counter (about 0.22 to 0.3 ns on arm64), and a body with nothing in it measures 0.00 ns because the optimizer deletes the loop. That counter is not subtracted: in a real body it overlaps the work.

**Helpers** (all in `Extensions/Testing`):

| | |
| --- | --- |
| `black_box(value)` | returns `value`, hidden from the optimizer. Wrap the inputs inside the loop (or the optimizer computes a pure call once) and the results (or it drops the work). It is a compiler intrinsic, an empty inline assembly block: no call and no instruction of its own, see [intrinsics](../language/intrinsics.md) |
| `set_bytes(b, n)` | the iteration handles `n` bytes: MB/s in the report |
| `set_items(b, n)` | it handles `n` items: items/s |
| `reset_timer(b)` | restart the clock and the allocation count |
| `@samples(N)` | samples for this benchmark |
| `@min_time(MS)` | spend MS sampling this benchmark instead of the default second |

`black_box` lives in `Extensions/Testing`, not in `Basic`: `Basic`'s exports mirror a fixed list, and a new name there could clash with a program's own `black_box`.

Other notes of `jacket test` work: `@only`, `@skip_on`, `@needs`, `@timeout(SECONDS)` (the default is `--timeout`, 120 s).

### Baselines

- `--save-baseline NAME` writes the JSON document below, with the raw samples, to `target/bench-baselines/NAME.json` (under the project folder).
- `--baseline NAME` compares each benchmark by name and adds the `change:` lines to the report, and a `baseline` object to the JSON. A benchmark the baseline lacks shows `new`. A baseline saved with another `schema` is refused; save it again.
- `--max-regression PCT` (needs `--baseline`) exits 1 when a benchmark regressed (verdict *regressed*, see above) and the change is more than PCT percent.

Both flags can be given at once: the comparison uses the old file, then it is overwritten.

### JSON document (`--format json`, and the baseline file)

One object on standard output; progress goes to standard error. `schema` changes when a field is removed or changes meaning (adding fields does not). Schema 2 replaced the median-based fields of schema 1.

```json
{
  "schema": 2,
  "jacket": "0.29.0",
  "os": "macos",
  "cpu": "arm64",
  "confidence_level": 0.95,
  "resamples": 100000,
  "noise_threshold_percent": 2.0,
  "significance_level": 0.05,
  "results": [
    {
      "name": "fnv1a",
      "file": "/abs/benches/hash.jai",
      "line": 7,
      "mode": "linear",
      "samples": 50,
      "step": 20198,
      "attempts": 1,
      "cpu": null,
      "estimator": "slope",
      "time_ns":   { "low": 41.24, "estimate": 41.30, "high": 41.37 },
      "mean_ns":   { "low": 41.27, "estimate": 41.33, "high": 41.40 },
      "median_ns": { "low": 41.26, "estimate": 41.31, "high": 41.40 },
      "mad_ns": 0.08,
      "r_squared": 0.9999,
      "overhead_ns": 61.0,
      "outliers": { "low_severe": 0, "low_mild": 2, "high_mild": 0, "high_severe": 1 },
      "bytes_per_iteration": 81,
      "items_per_iteration": 0,
      "mb_per_s": 1961.0,
      "items_per_s": null,
      "allocs_per_iteration": 0.0,
      "alloc_bytes_per_iteration": 0.0,
      "sample_iterations": [20198, 40396],
      "sample_ns": [834000, 1668000],
      "baseline": {
        "time_ns": 41.38,
        "change": { "low": -0.0041, "estimate": -0.0019, "high": 0.0005 },
        "change_percent": -0.19,
        "p_value": 0.12,
        "verdict": "no_change",
        "regression": null
      }
    }
  ]
}
```

Times are nanoseconds per iteration; `change` is a fraction (0.05 is 5% slower). `estimator` is `slope` or, for a flat run, `mean`; `r_squared` and `overhead_ns` are `null` for a flat run; `cpu` is the pinned core on Linux and `null` elsewhere. `verdict` is `improved`, `regressed`, `within_noise` or `no_change`. `baseline` is present only with `--baseline`; `regression` is `null` without `--max-regression`.

## How to change it

- The benchmark process: `stdlib/Extensions/Testing/bench.jai` (`run_bench`: warm-up, sampling, retry, pinning; `black_box`). The bench table is written by `registry_source` in `Discovery/module.jai`; the notes are checked in `bench_note_problem`. Command line of the program: `runner.jai`.
- Statistics: `crates/jacket/src/test/bench_stats.rs` (the fit, the bootstrap, outliers, the comparison). Report, baselines and exit status: `bench.rs`. Flags: `BenchArgs` in `cli.rs`.
- Gotcha: a benchmark is only as honest as its `black_box` calls. A pure call on known inputs is computed once by the optimizer and the loop measures nothing (it then reports about 0.3 ns). If a number looks too good, check what the optimizer made of the loop.
- Gotcha: the sampling schedule assumes the cost of one clock read pair is small next to a sample (about a millisecond). Do not make the shortest sample shorter than that.
- Gotcha: `count_allocations` resets temporary storage, so do not keep temp allocations across it in the runner.

## Configuration

| Flag | Default | |
| --- | --- | --- |
| `--samples N` | 50 | samples per benchmark (`@samples`) |
| `--warmup MS` | 100 | least warm-up; it goes on until the time is steady |
| `--measure-ms MS` | 1000 | time spent sampling (`@min_time`) |
| `--resamples N` | 100000 | bootstrap resamples |
| `--noise-threshold PCT` | 2 | a change smaller than this is noise |
| `--significance P` | 0.05 | a change counts below this p-value |
| `--save-baseline`, `--baseline`, `--max-regression` | | see Baselines |

And the selection and build options of `jacket test` (`FILTER`, `-E`, `--path`, `--format`, `--timeout`, `-os`/`-cpu`, flags after `--`). `[test] paths` in `jacket.toml` name the roots, as for tests. For steadier numbers close other programs first. On a Mac running other compiler builds (load average about 3) five runs of the shipped suite differed by 1 to 8% between the lowest and highest estimate (the quietest benches by 1 to 3%); an idle machine does better, and a single quiet run has a 95% interval of about 0.3%.

## Dependencies

The `Extensions/Testing` module and its Discovery plugin; `jaic` with the LLVM backend. Not covered yet: the `tools/bench` runtime workloads (`benchmarks/runtime`) compare whole programs across two compiler builds and stay as they are; `benchmarks/bench/` holds `@bench` versions of the Hash module's functions as the first suite.
