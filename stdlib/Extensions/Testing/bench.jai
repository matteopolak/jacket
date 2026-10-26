// Benchmarks: procedures with a `@bench` note, measured by `jacket bench`. A bench loops
// `b.iterations` times over the work it measures. The runner warms up until the time per
// iteration is steady, then times samples of k * d iterations for k = 1, 2, 3... and hands the
// raw (iterations, nanoseconds) pairs to jacket, which fits the line and reports the slope.
// See docs/tools/jacket-bench.md.
//
//     hashing :: (b: *Bench) {
//         text := "the quick brown fox";
//         set_bytes(b, text.count);
//
//         for 0..b.iterations - 1 {
//             black_box(get_hash(text));
//         }
//     } @bench @samples(40)

// Returns `value` and hides it from the optimizer: the compiler cannot know the result, and
// cannot drop the work that made it, so wrap what a bench computes and the inputs it must not
// see through. It is an intrinsic that costs no instruction: an empty assembly block takes the
// value and may change it (and any memory), see docs/language/intrinsics.md.
black_box :: (value: $T) -> T #intrinsic;

// Each iteration handles `count` bytes, so the report shows a rate in bytes per second.
set_bytes :: (b: *Bench, count: s64) {
    b.bytes_per_iteration = count;
}

// Each iteration handles `count` items (requests, elements), shown as items per second.
set_items :: (b: *Bench, count: s64) {
    b.items_per_iteration = count;
}

// Starts the measured time and the count of allocations again, for a bench that sets up its
// input before the loop.
reset_timer :: (b: *Bench) {
    b.started = current_time_monotonic();

    if b.counter {
        b.counter.allocations = 0;
        b.counter.bytes = 0;
    }
}

// One `@bench` the Discovery plugin found.
Bench_Case :: struct {
    name: string;
    body: (b: *Bench);
    // From `@samples(N)` and `@min_time(MS)`; 0 means the default.
    samples: s64;
    min_time_ms: s64;
}

DEFAULT_SAMPLES :: 50;
DEFAULT_WARMUP_MS :: 100;
DEFAULT_MEASURE_MS :: 1000;

// Settings of a run from the command line (`--samples`, `--warmup`, `--measure-ms`); 0 keeps
// what the bench or the defaults say.
Bench_Settings :: struct {
    samples: s64;
    warmup_ms: s64;
    measure_ms: s64;
}

#scope_module

Counter :: struct {
    parent: Allocator;
    allocations: s64;
    bytes: s64;
}

// Fewest samples a slow bench gets, and the span of the batches that settle the warm-up.
MIN_SAMPLES :: 10;
SETTLE_BATCHES :: 5;
SETTLE_BATCH_NS :: 10_000_000;
SETTLE_SPREAD :: 0.02;
MAX_WARMUP_FACTOR :: 10;
MAX_ATTEMPTS :: 3;
STEADY_QUALITY :: 0.99;
LONGEST_BATCH :: 4_000_000_000;

// The least time per iteration the runner believes, in nanoseconds: a body that took no
// measurable time (its loop was deleted) still gets a finite batch.
SHORTEST_NS :: 0.001;

run_bench :: (bench_case: *Bench_Case, settings: Bench_Settings) {
    runner_allocator = context.allocator;

    cpu := pin_to_cpu();

    warmup_ns := 1_000_000 * ifx settings.warmup_ms then settings.warmup_ms else DEFAULT_WARMUP_MS;
    measure_ms := ifx settings.measure_ms then settings.measure_ms else ifx bench_case.min_time_ms then bench_case.min_time_ms else DEFAULT_MEASURE_MS;
    measure_ns := cast(float64) measure_ms * 1_000_000.0;
    samples := ifx settings.samples then settings.samples else ifx bench_case.samples then bench_case.samples else DEFAULT_SAMPLES;

    b: Bench;

    per_iteration := warm_up(*b, bench_case, warmup_ns);

    // Sample k runs k * step iterations, so the total is step * samples * (samples + 1) / 2.
    // A bench so slow that one step would not fit gets equal samples (and fewer of them).
    linear := true;
    step: s64;
    fit := measure_ns / (per_iteration * cast(float64) (samples * (samples + 1) / 2));

    if fit >= 1.0 {
        step = cast(s64) (fit + 0.5);
    } else {
        linear = false;

        affordable := cast(s64) (measure_ns / per_iteration);
        if samples > affordable samples = affordable;
        if samples < MIN_SAMPLES samples = MIN_SAMPLES;

        step = affordable / samples;
        if step < 1 step = 1;
    }

    // On the heap: counting allocations resets temporary storage.
    iterations: [..] s64;
    nanoseconds: [..] s64;
    trial_iterations: [..] s64;
    trial_nanoseconds: [..] s64;

    defer array_free(iterations);
    defer array_free(nanoseconds);

    // A burst of load from another process spoils the samples it lands on. When the samples do
    // not lie on a line, take them again and keep the take that does.
    attempts := 0;
    quality: float64 = -1.0;

    while attempts < MAX_ATTEMPTS {
        attempts += 1;
        array_reset_keeping_memory(*trial_iterations);
        array_reset_keeping_memory(*trial_nanoseconds);

        for 1..samples {
            count := ifx linear then it * step else step;
            if count > LONGEST_BATCH count = LONGEST_BATCH;

            array_add(*trial_iterations, count);
            array_add(*trial_nanoseconds, time_batch(*b, bench_case, count));
        }

        trial_quality := ifx linear then line_quality(trial_iterations, trial_nanoseconds) else 1.0;

        if trial_quality > quality {
            quality = trial_quality;
            array_copy(*iterations, trial_iterations);
            array_copy(*nanoseconds, trial_nanoseconds);
        }

        if quality >= STEADY_QUALITY break;
    }

    array_free(trial_iterations);
    array_free(trial_nanoseconds);

    // The allocation count comes from one more batch whose allocator counts, so counting never
    // slows the timed ones.
    counter := count_allocations(*b, bench_case, step);

    json: String_Builder;
    json.allocator = temp;
    print_to_builder(*json, "{\"name\":\"%\",\"mode\":\"%\",\"samples\":%,\"step\":%,\"cpu\":%,\"attempts\":%", bench_case.name, ifx linear then "linear" else "flat", samples, step, cpu, attempts);
    print_to_builder(*json, ",\"bytes_per_iteration\":%,\"items_per_iteration\":%", b.bytes_per_iteration, b.items_per_iteration);
    print_to_builder(*json, ",\"allocs_per_iteration\":%,\"alloc_bytes_per_iteration\":%", per_unit(counter.allocations, step), per_unit(counter.bytes, step));
    append_list(*json, ",\"sample_iterations\":[", iterations);
    append_list(*json, "],\"sample_ns\":[", nanoseconds);
    append(*json, "]}");

    // The result goes to jacket as a record on standard output (see runner.jai).
    record: String_Builder;
    record.allocator = temp;
    append(*record, "\x02bench\x1f");
    append(*record, builder_to_string(*json,, temp));
    append(*record, "\x03");
    write_builder(*record);
}

#scope_file

#if OS == .LINUX {
    libc :: #system_library "libc";

    sched_getcpu :: () -> s32 #foreign libc;
    sched_setaffinity :: (pid: s32, size: u64, mask: *u64) -> s32 #foreign libc;
}

// Keeps the benchmark on the core it is on, so it is not moved (and its caches cooled) in the
// middle of a run. Returns that core, or -1 where the system gives no way to pin (macOS).
pin_to_cpu :: () -> s64 {
    #if OS == .LINUX {
        cpu := sched_getcpu();
        if cpu < 0 || cpu >= 1024 return -1;

        mask: [16] u64;
        mask[cpu / 64] = cast(u64) 1 << cast(u64) (cpu % 64);

        if sched_setaffinity(0, size_of([16] u64), *mask[0]) != 0 return -1;

        return cpu;
    }

    return -1;
}

// Runs the body until its time per iteration is steady and returns it, in nanoseconds.
// First batches that double find the order of magnitude, then batches of about 10 ms go on until
// the warm-up time has passed and the last five agree within 2%: caches, branch predictors and
// the clock speed have settled. A body that never settles stops at ten times the warm-up.
warm_up :: (b: *Bench, bench_case: *Bench_Case, warmup_ns: s64) -> float64 {
    iterations := 1;
    warm := 0;
    per_iteration: float64;

    while true {
        elapsed := time_batch(b, bench_case, iterations);

        warm += elapsed;
        per_iteration = longer_than_zero(elapsed, iterations);

        if elapsed >= 1_000_000 || iterations >= LONGEST_BATCH break;

        iterations *= 2;
    }

    recent: [SETTLE_BATCHES] float64;
    batches := 0;

    while warm < warmup_ns * MAX_WARMUP_FACTOR {
        iterations = cast(s64) (cast(float64) SETTLE_BATCH_NS / per_iteration);
        if iterations < 1 iterations = 1;
        if iterations > LONGEST_BATCH iterations = LONGEST_BATCH;

        elapsed := time_batch(b, bench_case, iterations);

        warm += elapsed;
        per_iteration = longer_than_zero(elapsed, iterations);

        recent[batches % SETTLE_BATCHES] = per_iteration;
        batches += 1;

        if batches >= SETTLE_BATCHES && warm >= warmup_ns {
            low := recent[0];
            high := recent[0];

            for recent {
                if it < low low = it;
                if it > high high = it;
            }

            if high - low <= SETTLE_SPREAD * low break;
        }
    }

    return per_iteration;
}

// How well the samples lie on a line through the origin: 1 minus the share of the variance that
// the line leaves, the R-squared that jacket reports.
line_quality :: (iterations: [..] s64, nanoseconds: [..] s64) -> float64 {
    sum_xx: float64 = 0.0;
    sum_xy: float64 = 0.0;
    sum_y: float64 = 0.0;

    for iterations {
        x := cast(float64) it;
        y := cast(float64) nanoseconds[it_index];

        sum_xx += x * x;
        sum_xy += x * y;
        sum_y += y;
    }

    slope := sum_xy / sum_xx;
    mean := sum_y / cast(float64) iterations.count;
    residual: float64 = 0.0;
    total: float64 = 0.0;

    for iterations {
        x := cast(float64) it;
        y := cast(float64) nanoseconds[it_index];

        residual += (y - slope * x) * (y - slope * x);
        total += (y - mean) * (y - mean);
    }

    if total <= 0.0 return 1.0;

    return 1.0 - residual / total;
}

longer_than_zero :: (elapsed: s64, iterations: s64) -> float64 {
    per_iteration := cast(float64) elapsed / cast(float64) iterations;
    if per_iteration < SHORTEST_NS return SHORTEST_NS;

    return per_iteration;
}

// Runs the body for `iterations` and gives the time it took, in nanoseconds. The clock is read
// twice per batch, not per iteration: whatever that and the call cost is the same for every
// batch, and the line jacket fits through the samples leaves it out.
time_batch :: (b: *Bench, bench_case: *Bench_Case, iterations: s64) -> s64 {
    reset_temporary_storage();
    b.iterations = iterations;
    b.counter = null;
    b.started = current_time_monotonic();
    bench_case.body(b);

    nanoseconds, ok := to_nanoseconds(current_time_monotonic() - b.started);
    if !ok nanoseconds = 0;

    return nanoseconds;
}

count_allocations :: (b: *Bench, bench_case: *Bench_Case, iterations: s64) -> Counter {
    reset_temporary_storage();

    counter := Counter.{ parent = context.allocator };

    b.iterations = iterations;
    b.counter = *counter;
    b.started = current_time_monotonic();

    new_context := context;
    new_context.allocator = .{ counting_proc, *counter };

    push_context new_context {
        bench_case.body(b);
    }

    b.counter = null;

    return counter;
}

counting_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    counter := cast(*Counter) allocator_data;

    if mode == .ALLOCATE || mode == .RESIZE {
        counter.allocations += 1;
        counter.bytes += size;
    }

    return counter.parent.proc(mode, size, old_size, old_memory, counter.parent.data);
}

append_list :: (json: *String_Builder, opening: string, values: [..] s64) {
    append(json, opening);

    for values {
        if it_index > 0 append(json, ",");
        print_to_builder(json, "%", it);
    }
}

// A number with three decimals and no exponent, which JSON and `string_to_float` agree on.
per_unit :: (total: s64, iterations: s64) -> string {
    value := cast(float64) total / cast(float64) iterations;
    thousandths := cast(s64) (value * 1000.0 + 0.5);

    return tprint("%.%%%", thousandths / 1000, (thousandths / 100) % 10, (thousandths / 10) % 10, thousandths % 10);
}
