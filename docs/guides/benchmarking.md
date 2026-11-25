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

    for b {
        slug := slugify(black_box(title));
        black_box(slug.count);
        free(slug);
    }
} @bench

long_title :: (b: *Bench) {
    title := "The Quick Brown Fox Jumps Over The Lazy Dog, Again And Again";
    set_bytes(b, title.count);

    for b {
        slug := slugify(black_box(title));
        black_box(slug.count);
        free(slug);
    }
} @bench @samples(30)
```

```console cwd=../examples/benchmarking/slugs cache
$ jacket bench short_title long_title
     Running short_title
short_title             time:   [124.6 128.1 131.9 ns]
                        R² 0.9651   median [122.6 123.9 132.5 ns]   allocs/iter 1.00   3 attempts
Found 3 outliers among 50 measurements (6.00%)
  3 (6.00%) high mild

     Running long_title
long_title              time:   [279.4 286.0 293.4 ns]
                        thrpt:  [204.5 209.8 214.8 MB/s]
                        R² 0.9827   median [277.3 285.8 298.4 ns]   allocs/iter 1.00   3 attempts
```

A benchmark takes a `*Bench`, or nothing, and wraps the work it measures in `for b { ... }`. The runner picks how many times the body runs and `jacket bench` divides by it. `break` and `continue` work as in any `for`.

## Writing a benchmark

Wrap what goes in and what comes out in `black_box`. The optimizer sees through a constant input and deletes work whose result nobody reads, and then the benchmark times an empty loop. A time of about 0.3 ns is the sign.

```jai
result := compute(black_box(input));
black_box(result);
```

Work that must happen once, before the loop, such as building the input, runs once per sample. The clock starts at the first iteration of `for b`, so that work is not timed:

```jai
many_titles :: (b: *Bench) {
    titles: [..] string;
    for 0..99 array_add(*titles, tprint("Title Number %", it));
    set_items(b, titles.count);

    for b {
        for titles {
            slug := slugify(it);
            black_box(slug.count);
            free(slug);
        }
    }
} @bench @min_time(500)
```

```console cwd=../examples/benchmarking/slugs cache
$ jacket bench many_titles
     Running many_titles
many_titles             time:   [12.22 12.61 13.03 µs]
                        thrpt:  [7.677 7.932 8.181 M/s]
                        R² 0.9619   median [11.76 12.06 12.41 µs]   allocs/iter 100.00   3 attempts
Found 3 outliers among 50 measurements (6.00%)
  1 (2.00%) high mild
  2 (4.00%) high severe
```

| Call or note | Effect |
| --- | --- |
| `black_box(value)` | returns `value`, opaque to the optimizer |
| `set_bytes(b, n)` | `n` bytes are processed per iteration: adds a `thrpt` line in MB/s |
| `set_items(b, n)` | `n` items per iteration: `thrpt` in items per second |
| `pause_timer(b)`, `resume_timer(b)` | stop and restart the clock and the allocation count inside the loop, for per-iteration setup or teardown |
| `start_timer(b)`, `stop_timer(b)`, `iteration_count(b)` | for work that other threads do: the bench starts the clock, waits, stops it, and hands `iteration_count(b)` iterations to the workers instead of using `for b` |
| `@samples(N)` | measurements to take, instead of 50 |
| `@min_time(MS)` | time to spend measuring, instead of 1000 ms |
| `@only(...)`, `@skip_on(...)`, `@needs(...)`, `@timeout(SECONDS)` | as for [tests](testing.md#options) |

Benchmarks can also sit in `tests/*.jai` next to your tests. `jacket bench` runs only the `@bench` procedures, and `jacket test` doesn't run them.

## Reading the output

```console cwd=../examples/benchmarking/slugs cache
$ jacket bench --exact long_title
     Running long_title
long_title              time:   [272.7 281.6 292.1 ns]
                        thrpt:  [205.4 213.0 220.0 MB/s]
                        R² 0.9793   median [271.1 281.0 290.6 ns]   allocs/iter 1.00   3 attempts
```

- `time` is the 95% confidence interval for the time of one iteration: low, best estimate, high. A narrow interval means a steady result.
- `thrpt` appears when you called `set_bytes` or `set_items`.
- `R²` says how well the samples lie on a straight line, as they should when time grows with the number of iterations. Below 0.99 the runner measures again, up to three times (`3 attempts`). A low value on a machine doing other work is normal; close other programs and run again.
- `median` is the middle sample, a check on the estimate.
- `allocs/iter` counts allocations per iteration, in the loop only.
- `clock paused per iteration` means the benchmark calls `pause_timer`. Each pause costs two reads of the clock, so use it only when an iteration is long next to that (a microsecond or more).
- Outliers are measurements far from the rest. A few high ones are usual and are already accounted for.

Pick benchmarks by name like tests: `jacket bench long`, `jacket bench --exact long_title`, `jacket bench -E 'file(**/slugify.jai)'`. `--list` prints them without running anything.

## Comparing with a baseline

Save the numbers before you change something:

```console cwd=../examples/benchmarking/slugs cache
$ jacket bench --save-baseline main
     Running short_title
short_title             time:   [115.2 117.5 119.8 ns]
                        R² 0.9850   median [112.5 116.8 119.9 ns]   allocs/iter 1.00   3 attempts
Found 3 outliers among 50 measurements (6.00%)
  3 (6.00%) high mild

     Running long_title
long_title              time:   [280.0 287.3 294.6 ns]
                        thrpt:  [203.7 208.8 214.3 MB/s]
                        R² 0.9866   median [269.6 281.2 295.5 ns]   allocs/iter 1.00   3 attempts

     Running many_titles
many_titles             time:   [11.84 12.12 12.42 µs]
                        thrpt:  [8.054 8.253 8.447 M/s]
                        R² 0.9823   median [11.37 11.52 11.94 µs]   allocs/iter 100.00   3 attempts
Found 2 outliers among 50 measurements (4.00%)
  2 (4.00%) high mild

jacket: saved the baseline target/bench-baselines/main.json
```

Change the code, then compare. Here `slugify` was changed to copy every character into a string of its own:

```console cwd=../examples/benchmarking/slower/slugs cache
$ jacket bench --baseline main long_title
     Running long_title
long_title              time:   [1.465 1.507 1.549 µs]
                        thrpt:  [38.73 39.81 40.95 MB/s]
                        R² 0.9831   median [1.438 1.482 1.533 µs]   allocs/iter 49.00   3 attempts
                        change: [+438.84% +460.68% +481.44%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 2 outliers among 30 measurements (6.67%)
  2 (6.67%) high mild
```

The `change` line is the interval for the change against the baseline, with a p-value. The verdict is one of:

| Verdict | Meaning |
| --- | --- |
| Performance has regressed / improved | p is below 0.05 and the whole interval lies beyond the noise threshold (2%) |
| Change within noise threshold | the change is real but smaller than the threshold |
| No change in performance detected | p is above 0.05 |

The verdict is only as good as the machine is quiet. Here is the same benchmark on unchanged code, run while another program kept several cores busy:

```console cwd=../examples/benchmarking/unchanged/slugs cache
$ jacket bench --baseline main long_title
     Running long_title
long_title              time:   [278.6 289.3 300.0 ns]
                        thrpt:  [200.0 207.4 215.4 MB/s]
                        R² 0.9725   median [272.8 284.5 299.6 ns]   allocs/iter 1.00   3 attempts
                        change: [+2.72% +7.61% +12.45%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 30 measurements (3.33%)
  1 (3.33%) high mild
```

Close other programs before you compare, and compare runs from the same machine. A benchmark missing from the baseline is shown as `new`. A baseline written by another version of the format is refused; save it again.

To fail a build on a slowdown, add a limit. The exit status is 1 when a benchmark regressed by more than that many percent:

```console cwd=../examples/benchmarking/slower/slugs cache
$ jacket bench --baseline main --max-regression 10 long_title
     Running long_title
long_title              time:   [1.574 1.637 1.699 µs]
                        thrpt:  [35.31 36.66 38.11 MB/s]
                        R² 0.9731   median [1.503 1.568 1.689 µs]   allocs/iter 49.00   3 attempts
                        change: [+480.79% +508.78% +536.77%] (p = 0.00 < 0.05)
                        Performance has regressed.

jacket: regression: long_title is +508.8% slower than the baseline (limit 10%)
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
| `name`, `file`, `line` | the benchmark; `file` is relative to the project folder |
| `time_ns`, `mean_ns`, `median_ns` | `{low, estimate, high}`, in nanoseconds per iteration |
| `r_squared`, `outliers` | fit quality; counts as `high_mild`, `high_severe`, `low_mild`, `low_severe` |
| `mb_per_s`, `items_per_s` | throughput, when set |
| `allocs_per_iteration`, `alloc_bytes_per_iteration` | allocations per iteration |
| `paused` | the benchmark pauses its clock inside the loop |
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
