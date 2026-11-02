# Benchmarking

A benchmark is a procedure with a `@bench` note. `jacket bench` builds it optimized, times it in its own process and reports the time per iteration with a confidence interval. It can also compare a run with an earlier one and tell you whether the difference is real.

## A first benchmark

This benchmarks `slugify` from the library in [Testing](testing.md). Put benchmarks in a `benches/` folder next to `module.jai`:

```jai
// slugs/benches/slugify.jai
#import "Basic";
#import "Extensions/Testing";
#import "slugs";

short_title :: (b: *Bench) {
    title := "Hello, World!";

    for 0..b.iterations - 1 {
        slug := slugify(black_box(title));
        black_box(slug.count);
        free(slug);
    }
} @bench

long_title :: (b: *Bench) {
    title := "The Quick Brown Fox Jumps Over The Lazy Dog, Again And Again";
    set_bytes(b, title.count);

    for 0..b.iterations - 1 {
        slug := slugify(black_box(title));
        black_box(slug.count);
        free(slug);
    }
} @bench @samples(30)
```

```sh
jacket bench
```

```text
     Running short_title
short_title             time:   [152.8 165.1 177.7 ns]
                        R² 0.8492   median [161.5 169.9 180.2 ns]   allocs/iter 1.00   3 attempts

     Running long_title
long_title              time:   [275.4 279.4 284.5 ns]
                        thrpt:  [210.9 214.7 217.9 MB/s]
                        R² 0.9919   median [276.8 281.6 291.6 ns]   allocs/iter 1.00
Found 1 outliers among 30 measurements (3.33%)
  1 (3.33%) high mild
```

A benchmark takes a `*Bench`, or nothing, and loops `b.iterations` times. The runner picks that number; you do the work inside the loop and `jacket bench` divides by it.

## Writing a benchmark

Wrap what goes in and what comes out in `black_box`. The optimizer sees through a constant input and deletes work whose result nobody reads, and then the benchmark times an empty loop. A time of about 0.3 ns is the sign.

```jai
result := compute(black_box(input));
black_box(result);
```

Work that must happen once, before the loop, such as building the input, runs once per sample. Call `reset_timer(b)` after it to leave it out of the time:

```jai
many_titles :: (b: *Bench) {
    titles: [..] string;
    for 0..99 array_add(*titles, tprint("Title Number %", it));
    reset_timer(b);
    set_items(b, titles.count);

    for 0..b.iterations - 1 {
        for titles {
            slug := slugify(it);
            black_box(slug.count);
            free(slug);
        }
    }
} @bench @min_time(500)
```

```text
many_titles             time:   [14.10 15.05 16.04 µs]
                        thrpt:  [6.236 6.644 7.092 M/s]
                        R² 0.8756   median [12.22 13.22 14.87 µs]   allocs/iter 100.00   3 attempts
```

| Call or note | Effect |
| --- | --- |
| `black_box(value)` | returns `value`, opaque to the optimizer |
| `set_bytes(b, n)` | `n` bytes are processed per iteration: adds a `thrpt` line in MB/s |
| `set_items(b, n)` | `n` items per iteration: `thrpt` in items per second |
| `reset_timer(b)` | restart the clock and the allocation count |
| `@samples(N)` | measurements to take, instead of 50 |
| `@min_time(MS)` | time to spend measuring, instead of 1000 ms |
| `@only(...)`, `@skip_on(...)`, `@needs(...)`, `@timeout(SECONDS)` | as for [tests](testing.md#options) |

Benchmarks can also sit in `tests/*.jai` next to your tests. `jacket bench` runs only the `@bench` procedures, and `jacket test` doesn't run them.

## Reading the output

```text
long_title              time:   [275.4 279.4 284.5 ns]
                        thrpt:  [210.9 214.7 217.9 MB/s]
                        R² 0.9919   median [276.8 281.6 291.6 ns]   allocs/iter 1.00
Found 1 outliers among 30 measurements (3.33%)
  1 (3.33%) high mild
```

- `time` is the 95% confidence interval for the time of one iteration: low, best estimate, high. A narrow interval means a steady result.
- `thrpt` appears when you called `set_bytes` or `set_items`.
- `R²` says how well the samples lie on a straight line, as they should when time grows with the number of iterations. Below 0.99 the runner measures again, up to three times (`3 attempts`). A low value on a machine doing other work is normal; close other programs and run again.
- `median` is the middle sample, a check on the estimate.
- `allocs/iter` counts allocations per iteration.
- Outliers are measurements far from the rest. A few high ones are usual and are already accounted for.

Pick benchmarks by name like tests: `jacket bench long`, `jacket bench --exact long_title`, `jacket bench -E 'file(**/slugify.jai)'`. `--list` prints them without running anything.

## Comparing with a baseline

Save the numbers before you change something:

```sh
jacket bench --save-baseline main
```

```text
jacket: saved the baseline target/bench-baselines/main.json
```

Change the code, then compare:

```sh
jacket bench --baseline main
```

```text
long_title              time:   [1.135 1.151 1.167 µs]
                        thrpt:  [51.40 52.13 52.85 MB/s]
                        R² 0.9964   median [1.124 1.133 1.150 µs]   allocs/iter 42.00
                        change: [+202.91% +234.79% +281.44%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

The `change` line is the interval for the change against the baseline, with a p-value. The verdict is one of:

| Verdict | Meaning |
| --- | --- |
| Performance has regressed / improved | p is below 0.05 and the whole interval lies beyond the noise threshold (2%) |
| Change within noise threshold | the change is real but smaller than the threshold |
| No change in performance detected | p is above 0.05 |

```text
long_title              time:   [304.6 318.1 330.5 ns]
                        change: [-25.95% -14.50% -1.99%] (p = 0.47 > 0.05)
                        No change in performance detected.
```

A benchmark missing from the baseline is shown as `new`. A baseline written by another version of the format is refused; save it again.

To fail a build on a slowdown, add a limit. The exit status is 1 when a benchmark regressed by more than that many percent:

```sh
jacket bench --baseline main --max-regression 10
```

```text
jacket: regression: long_title is +234.8% slower than the baseline (limit 10%)
```

Commit baselines if you want CI to compare against them, or save one in the job before the change under test. Numbers from different machines don't compare; keep the baseline and the run on the same one.

## JSON

`--format json` writes one JSON object to standard output (progress goes to standard error), for scripts:

```sh
jacket bench --format json long > result.json
```

The top level holds `schema`, `os`, `cpu`, `confidence_level`, `noise_threshold_percent` and `results`. Each result has:

| Field | Value |
| --- | --- |
| `name`, `file`, `line` | the benchmark |
| `time_ns`, `mean_ns`, `median_ns` | `{low, estimate, high}`, in nanoseconds per iteration |
| `r_squared`, `outliers` | fit quality; counts as `high_mild`, `high_severe`, `low_mild`, `low_severe` |
| `mb_per_s`, `items_per_s` | throughput, when set |
| `allocs_per_iteration`, `alloc_bytes_per_iteration` | allocations per iteration |
| `sample_iterations`, `sample_ns` | the raw samples |
| `baseline` | with `--baseline`: `change` (a fraction), `change_percent`, `p_value`, `verdict` |

A saved baseline is this same file.

## Options

| Flag | Default | |
| --- | --- | --- |
| `--samples N` | 50 | measurements per benchmark |
| `--measure-ms MS` | 1000 | time spent measuring each |
| `--warmup MS` | 100 | least warm-up before measuring |
| `--noise-threshold PCT` | 2 | changes smaller than this are noise |
| `--significance P` | 0.05 | p-value cutoff |
| `--resamples N` | 100000 | bootstrap resamples for the intervals |

Selection and build flags are those of [jacket test](testing.md#running-tests). A flag after `--` that picks an optimization level replaces the default one. The exit status is 0, 1 when a benchmark failed or regressed past `--max-regression`, and 2 for a usage error, a missing baseline or a build that doesn't compile. Every flag is in the [command-line reference](../reference/tools.md#benchmarking).
