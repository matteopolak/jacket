// Benchmarks: procedures with a `@bench` note, measured by `jacket bench`. A bench loops
// `b.iterations` times over the work it measures; the runner picks that number so one batch
// takes about 10 ms, times many batches and reports the median and the spread. See
// docs/tools/jacket-bench.md.
//
//     hashing :: (b: *Bench) {
//         text := "the quick brown fox";
//         set_bytes(b, text.count);
//
//         for 0..b.iterations - 1 {
//             black_box(get_hash(text));
//         }
//     } @bench @samples(40)

// Hides `value` from the optimizer: the result is not known at compile time and the work that
// made it cannot be dropped as unused. Wrap what a bench computes, and the inputs it should not
// see through.
black_box :: (value: $T) -> T {
    copy := value;
    keeper(*copy, size_of(T));

    return copy;
}

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

// The least time per iteration the runner believes, in nanoseconds: a body that took no
// measurable time still gets a finite batch.
SHORTEST_NS :: 0.001;

DEFAULT_SAMPLES :: 30;
DEFAULT_WARMUP_MS :: 100;
DEFAULT_SAMPLE_MS :: 10;

// Settings of a run from the command line (`--samples`, `--warmup`, `--sample-ms`); 0 keeps
// what the bench or the defaults say.
Bench_Settings :: struct {
    samples: s64;
    warmup_ms: s64;
    sample_ms: s64;
}

#scope_module

Counter :: struct {
    parent: Allocator;
    allocations: s64;
    bytes: s64;
}

// What `black_box` calls. The compiler has to assume that a call through this pointer reads
// and changes the memory it is given, because it cannot tell which of two procedures runs:
// `settle` is the one that does (`run_bench` may switch to `inspect`, which reads every byte).
// A direct call would not do: the optimizer sees that `settle` never reads the memory and
// drops the stores that fill it.
keeper: (pointer: *void, size: s64) = settle;

// Where `settle` leaves the pointer, and what `inspect` computes; `run_bench` reads both, so
// the stores to them are not dead.
sink: *void;
sink_sum: s64;

settle :: no_inline (pointer: *void, size: s64) {
    sink = pointer;

    if pointer == cast(*void) 1 {
        cast(*u8, pointer).* = 0;
    }
}

inspect :: no_inline (pointer: *void, size: s64) {
    sum := 0;
    for 0..size - 1 sum += (cast(*u8, pointer) +it).*;

    sink_sum = sum;
    sink = pointer;

    if pointer == cast(*void) 1 {
        cast(*u8, pointer).* = 0;
    }
}

run_bench :: (bench_case: *Bench_Case, settings: Bench_Settings) {
    runner_allocator = context.allocator;

    sample_ns := 1_000_000 * ifx settings.sample_ms then settings.sample_ms else DEFAULT_SAMPLE_MS;
    warmup_ns := 1_000_000 * ifx settings.warmup_ms then settings.warmup_ms else DEFAULT_WARMUP_MS;
    samples := ifx settings.samples then settings.samples else ifx bench_case.samples then bench_case.samples else DEFAULT_SAMPLES;

    // Enough samples to measure for `@min_time` in total.
    if bench_case.min_time_ms {
        needed := (bench_case.min_time_ms * 1_000_000 + sample_ns - 1) / sample_ns;
        if needed > samples samples = needed;
    }

    b: Bench;

    // Warm up with batches that double, which also finds how long one iteration takes.
    iterations := 1;
    per_iteration: float64 = 0.0;
    warm := 0;

    while true {
        elapsed := time_batch(*b, bench_case, iterations);

        warm += elapsed;
        per_iteration = cast(float64) elapsed / cast(float64) iterations;

        // A batch that is already long enough is a good estimate.
        if warm >= warmup_ns && elapsed >= sample_ns / 4 break;
        if warm >= warmup_ns * 4 break;

        iterations *= 2;
    }

    if per_iteration < SHORTEST_NS per_iteration = SHORTEST_NS;

    batch := cast(s64) (cast(float64) sample_ns / per_iteration);
    if batch < 1 batch = 1;

    // On the heap: counting allocations resets temporary storage.
    times: [..] float64;

    defer array_free(times);

    for 0..samples - 1 {
        elapsed := time_batch(*b, bench_case, batch);
        array_add(*times, cast(float64) elapsed / cast(float64) batch);
    }

    // The allocation count comes from one more batch whose allocator counts, so counting never
    // slows the timed ones.
    counter := count_allocations(*b, bench_case, batch);

    sort_floats(times);
    median := middle(times);

    deviations: [..] float64;

    defer array_free(deviations);
    for times array_add(*deviations, absolute(it - median));

    sort_floats(deviations);

    json: String_Builder;
    json.allocator = temp;
    print_to_builder(*json, "{\"name\":\"%\",\"samples\":%,\"iterations\":%", bench_case.name, samples, batch);
    print_to_builder(*json, ",\"median_ns\":%,\"mad_ns\":%", fixed(median), fixed(middle(deviations)));
    print_to_builder(*json, ",\"min_ns\":%,\"max_ns\":%", fixed(times[0]), fixed(times[times.count - 1]));
    print_to_builder(*json, ",\"p5_ns\":%,\"p95_ns\":%", fixed(percentile(times, 5)), fixed(percentile(times, 95)));
    print_to_builder(*json, ",\"bytes_per_iteration\":%,\"items_per_iteration\":%", b.bytes_per_iteration, b.items_per_iteration);
    print_to_builder(*json, ",\"allocs_per_iteration\":%,\"alloc_bytes_per_iteration\":%}", fixed(cast(float64) counter.allocations / cast(float64) batch), fixed(cast(float64) counter.bytes / cast(float64) batch));

    // Never true, but the compiler cannot know it (see `keeper`).
    if sink == cast(*void) 1 || sink_sum == 1 {
        keeper = inspect;
    }

    // The result goes to jacket as a record on standard output (see runner.jai).
    record: String_Builder;
    record.allocator = temp;
    append(*record, "\x02bench\x1f");
    append(*record, builder_to_string(*json,, temp));
    append(*record, "\x03");
    write_builder(*record);
}

#scope_file

// Runs the body for `iterations` and gives the time it took, in nanoseconds.
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

sort_floats :: (values: [..] float64) {
    // Insertion sort: a run has tens of samples.
    for 1..values.count - 1 {
        value := values[it];
        j := it - 1;

        while j >= 0 && values[j] > value {
            values[j + 1] = values[j];
            j -= 1;
        }

        values[j + 1] = value;
    }
}

// The median of sorted values.
middle :: (values: [..] float64) -> float64 {
    count := values.count;
    if count == 0 return 0;
    if count % 2 == 1 return values[count / 2];

    return (values[count / 2 - 1] + values[count / 2]) / 2;
}

// The value `percent` percent of the way through sorted values.
percentile :: (values: [..] float64, percent: s64) -> float64 {
    if values.count == 0 return 0;

    position := cast(float64) (values.count - 1) * cast(float64) percent / 100.0;
    low := cast(s64) position;
    high := min(low + 1, values.count - 1);
    fraction := position - cast(float64) low;

    return values[low] + (values[high] - values[low]) * fraction;
}

absolute :: (value: float64) -> float64 {
    return ifx value < 0 then -value else value;
}

// A number with three decimals and no exponent, which JSON and `string_to_float` agree on.
fixed :: (value: float64) -> string {
    if value < 0 return tprint("-%", fixed(-value));

    thousandths := cast(s64) (value * 1000.0 + 0.5);

    return tprint("%.%%%", thousandths / 1000, (thousandths / 100) % 10, (thousandths / 10) % 10, thousandths % 10);
}
