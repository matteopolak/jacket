// Run with jacket bench --path stdlib/Extensions/Json.
#import "Extensions/Testing";
#import "Basic";
#import "File";
#import "Random";
#import "Flat_Pool";
#import "Extensions/Environment";
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

Floats :: struct {
    values: [..] float64;
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

    defer free_buffers(*builder);

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

// About `megabytes` MB of longitudes and latitudes with up to 17 significant digits, as in canada.json.
generate_floats :: (megabytes: int) -> string {
    random_seed(7);

    builder: String_Builder;

    defer free_buffers(*builder);

    append(*builder, "{\"values\": [");

    first := true;

    while builder_string_length(*builder) < megabytes * 1024 * 1024 {
        if !first append(*builder, ", ");

        first = false;

        value := (cast(float64) random_get() / 4294967296.0) * 360.0 - 180.0;

        encoded := to_json(value);

        append(*builder, encoded);
        free(encoded);
    }

    append(*builder, "]}\n");

    return builder_to_string(*builder);
}

json_case :: (b: *Bench, input: int, mode: int, encode: bool, $T: Type) {
    text: string;
    if input == 0 text = generate(4);
    else if input == 1 text = generate_floats(2);
    else {
        base := output_directory();

        name := string.["twitter.json", "canada.json", "citm_catalog.json"][input - 2];
        ok: bool;
        text, ok = read_entire_file(tprint("%/json-bench-data/%", base, name));
        assert(ok, "run `PYTHONPATH=tools python3 tools/portable/build_portable.py bench` first");
    }

    defer free(text);
    pool: Flat_Pool;

    init(*pool, 1024 * 1024 * 1024);
    defer fini(*pool);
    json_use_simd = mode != 0;
    json_index_min_size = ifx mode == 2 then 0 else 1 << 40;

    defer {
        json_use_simd = true;
        json_index_min_size = 256;
    }

    value, error := from_json(T, text);

    assert(error.kind == .NONE);
    defer free_json(*value);
    encoded := to_json(value);

    defer free(encoded);
    set_bytes(b, ifx encode then encoded.count else text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        {
            push_allocator(flat_pool_allocator_proc, *pool);

            if encode {
                result := to_json(black_box(value));
                black_box(result);
            } else {
                result, failure := from_json(T, black_box(text));
                assert(failure.kind == .NONE);
                black_box(result);
            }
        }

        reset(*pool);
    }
}

// Expand the input/type/mode matrix into individually selectable benchmarks.
json_benches :: () -> string {
    out: String_Builder;

    for input: 0..4 for mode : 0..2 for encoding : 0..1 for typed : 0..1 {
        if typed == 1 && input != 0 continue;

        type_name := ifx input == 1 then "Floats" else ifx typed == 1 then "Catalog" else "Json_Value";
        name := string.["catalog", "floats", "twitter", "canada", "citm"][input];
        mode_name := string.["scalar", "vector", "indexed"][mode];
        operation := ifx encoding == 1 then "encode" else "decode";

        print(*out, "%_%_%_% :: (b: *Bench) { json_case(b, %, %, %, %); } @bench\n",
            name, type_name, mode_name, operation, input, mode, encoding == 1, type_name);
    }

    return builder_to_string(*out);
}

#insert #run json_benches();
