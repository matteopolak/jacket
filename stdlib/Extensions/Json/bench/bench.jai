// Throughput of the JSON module with the scalar paths and with the vector paths (the structural
// index and the vector string scans), in MB/s of JSON text.
//
//     jaic build -O2 stdlib/Extensions/Json/bench/bench.jai -o /tmp/json-bench
//     /tmp/json-bench                        # a generated 4 MB document of mixed content
//     /tmp/json-bench twitter.json canada.json citm_catalog.json
//
// Files given on the command line are measured as untyped documents (`from_json(Json_Value, ...)`, then
// to_json of the value). The generated document is also decoded into and encoded from a typed
// struct. Real-world files: tools/json_bench_data.py downloads simdjson's examples.

#import "Basic";
#import "String";
#import "File";
#import "Random";
#import "Flat_Pool";
#import "Extensions/Json";

Address :: struct {
    street: string;
    city: string;
    zip: int;
}

Item :: struct {
    id: int;
    name: string;
    description: string;
    price: float64;
    ratio: float32;
    active: bool;
    tags: [..] string;
    address: Address;
    scores: [..] int;
}

Catalog :: struct {
    items: [..] Item;
}

WORDS :: string.["alpha", "beta", "gamma", "delta", "épsilon", "日本語", "line\\nbreak", "quote\\\"d", "plain text that is a little longer than the rest, with spaces", "x"];

pick :: (count: int) -> int {
    return cast(int) (random_get() % cast(u64) count);
}

sentence :: (builder: *String_Builder, words: int) {
    for 1..words {
        append(builder, WORDS[pick(WORDS.count)]);
        append(builder, " ");
    }
}

// A catalog of about `megabytes` MB, as JSON text with some pretty-printing whitespace.
generate :: (megabytes: int) -> string {
    random_seed(42);

    builder: String_Builder;

    append(*builder, "{\n  \"items\": [\n");

    first := true;

    while builder_string_length(*builder) < megabytes * 1024 * 1024 {
        if !first append(*builder, ",\n");

        first = false;

        append(*builder, "    {\"id\": ");
        print(*builder, "%", pick(1000000));
        append(*builder, ", \"name\": \"");
        sentence(*builder, 2);
        append(*builder, "\", \"description\": \"");
        sentence(*builder, 12);
        append(*builder, "\", \"price\": ");
        print(*builder, "%.%", pick(1000), pick(100));
        append(*builder, ", \"ratio\": 0.");
        print(*builder, "%", pick(10000));
        append(*builder, ", \"active\": ");
        append(*builder, ifx pick(2) then "true" else "false");
        append(*builder, ", \"tags\": [\"a\", \"bb\", \"ccc\"], \"address\": {\"street\": \"");
        sentence(*builder, 2);
        append(*builder, "\", \"city\": \"Springfield\", \"zip\": ");
        print(*builder, "%", 10000 + pick(80000));
        append(*builder, "}, \"scores\": [");

        for i: 0..pick(12) {
            if i append(*builder, ", ");
            print(*builder, "%", pick(100000));
        }

        append(*builder, "]}");
    }

    append(*builder, "\n  ]\n}\n");

    return builder_to_string(*builder);
}

seconds :: () -> float64 {
    return to_float64_seconds(current_time_monotonic());
}

// Runs `work` until about half a second has passed, with the pool emptied between runs, and
// returns the best time of a single run.
measure :: (pool: *Flat_Pool, work: () -> void) -> float64 {
    best: float64 = 1.0e30;
    started := seconds();
    runs := 0;

    while runs < 3 || (seconds() - started < 0.5 && runs < 200) {
        begin := seconds();

        {
            push_allocator(flat_pool_allocator_proc, pool);
            work();
        }

        elapsed := seconds() - begin;

        if elapsed < best best = elapsed;

        reset(pool);
        runs += 1;
    }

    return best;
}

number :: (value: float64, digits: int) -> string {
    return tprint("%", FormatFloat.{ value = value, trailing_width = digits });
}

report :: (what: string, megabytes: float64, scalar: float64, scan: float64, indexed: float64) {
    print("  % scalar % MB/s, vector % (x%), vector + index % (x%)\n", what, number(megabytes / scalar, 0), number(megabytes / scan, 0), number(scalar / scan, 2), number(megabytes / indexed, 0), number(scalar / indexed, 2));
}

bench_text: string;
bench_value: Json_Value;
bench_catalog: Catalog;
bench_encoded: string;

parse_work :: () {
    _, error := from_json(Json_Value, bench_text);
    assert(error.kind == .NONE);
}

encode_value_work :: () {
    encoded := to_json(bench_value);
    assert(encoded.count > 0);
}

decode_work :: () {
    _, error := from_json(Catalog, bench_text);
    assert(error.kind == .NONE);
}

encode_work :: () {
    encoded := to_json(bench_catalog);
    assert(encoded.count > 0);
}

best_of_two :: (pool: *Flat_Pool, work: () -> void) -> float64 {
    first := measure(pool, work);
    second := measure(pool, work);

    return ifx first < second then first else second;
}

compare :: (pool: *Flat_Pool, what: string, megabytes: float64, work: () -> void) {
    json_use_simd = false;
    scalar := best_of_two(pool, work);

    json_use_simd = true;
    json_index_min_size = 1 << 40;
    scan := best_of_two(pool, work);

    json_index_min_size = 0;
    indexed := best_of_two(pool, work);

    report(what, megabytes, scalar, scan, indexed);
}

bench :: (pool: *Flat_Pool, label: string, text: string, typed: bool) {
    bench_text = text;
    megabytes := cast(float64) text.count / (1024.0 * 1024.0);

    print("%: % bytes\n", label, text.count);

    json_index_min_size = 0;

    compare(pool, "from_json (untyped)", megabytes, parse_work);

    json_use_simd = false;
    value, error := from_json(Json_Value, text);

    assert(error.kind == .NONE);
    bench_value = value;
    bench_encoded = to_json(value);

    compare(pool, "to_json (Json_Value)", cast(float64) bench_encoded.count / (1024.0 * 1024.0), encode_value_work);

    json_use_simd = true;
    again := to_json(value);

    assert(again == bench_encoded, "the vector encoder must give the same text");

    if typed {
        compare(pool, "from_json (Catalog)", megabytes, decode_work);

        json_use_simd = false;
        catalog, error2 := from_json(Catalog, text);

        assert(error2.kind == .NONE);
        bench_catalog = catalog;

        compare(pool, "to_json (Catalog)", cast(float64) to_json(catalog).count / (1024.0 * 1024.0), encode_work);
    }

}

main :: () {
    pool: Flat_Pool;

    init(*pool, 1024 * 1024 * 1024);

    arguments := get_command_line_arguments();

    if arguments.count < 2 {
        text := generate(4);
        bench(*pool, "generated catalog", text, true);
    } else {
        for 1..arguments.count - 1 {
            text, ok := read_entire_file(arguments[it]);

            if !ok {
                print("cannot read %\n", arguments[it]);
                continue;
            }

            bench(*pool, arguments[it], text, false);
        }
    }
}
