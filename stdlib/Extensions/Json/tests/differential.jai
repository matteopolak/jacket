// The vector paths (the structural index, the vector string scans) must give the same result
// as the scalar ones for every text: the same value, or the same error down to its offset,
// line, column and path. Checked on the JSONTestSuite cases and on random documents with random
// damage, including strings with long runs of backslashes and quotes that straddle the
// 64-byte blocks of the index.
//
//     jaic run stdlib/Extensions/Json/tests/differential.jai

#import "Basic";
#import "Random";
#import "String";
#import "Extensions/Json";

#load "data/suite_cases.jai";

failures := 0;

check :: (condition: bool, what: string) {
    if condition return;
    print("FAIL: %\n", what);
    failures += 1;
}

Outcome :: struct {
    ok: bool;
    text: string; // the value re-encoded, or the error's message
    offset: int;
    line: int;
    column: int;
    path: string;
}

run :: (text: string, simd: bool) -> Outcome {
    json_use_simd = simd;
    json_index_min_size = 0;

    outcome: Outcome;
    value, error := parse_json(text);

    if error.kind == .NONE {
        outcome.ok = true;
        outcome.text = to_json(value);
    } else {
        outcome.text = tprint("%: %", error.kind, error.message);
        outcome.offset = error.offset;
        outcome.line = error.line;
        outcome.column = error.column;
        outcome.path = error.path;
    }

    return outcome;
}

compared := 0;

same :: (text: string, label: string) {
    a := run(text, false);
    b := run(text, true);

    compared += 1;

    equal := a.ok == b.ok && a.text == b.text && a.offset == b.offset && a.line == b.line && a.column == b.column && a.path == b.path;

    if !equal {
        shown := ifx text.count > 120 then slice(text, 0, 120) else text;
        check(false, tprint("% differs: scalar %/% at %, simd %/% at % in %", label, a.ok, a.text, a.offset, b.ok, b.text, b.offset, shown));
    }

    // The typed decoder reads through the same reader.
    json_use_simd = false;
    typed_a, error_a := from_json([..] Fuzz_Target, text);

    json_use_simd = true;
    typed_b, error_b := from_json([..] Fuzz_Target, text);

    check(error_a.kind == error_b.kind && error_a.offset == error_b.offset && error_a.path == error_b.path && error_a.message == error_b.message, tprint("% typed errors differ", label));

    if error_a.kind == .NONE check(to_json(typed_a) == to_json(typed_b), tprint("% typed values differ", label));
}

Fuzz_Target :: struct {
    name: string;
    id: int;
    score: float64;
    tags: [..] string;
    ok: bool;
}

pick :: (count: int) -> int {
    return cast(int) (random_get() % cast(u64) count);
}

PIECES :: string.["a", "b", " ", "xyz", "\\\"", "\\\\", "\\n", "\\/", "\\u00e9", "\\ud83d\\ude00", "é", "日本", "😀", "\\\\\\\\\\\\", "\\\\\\\\\\\\\\\"", "\\\"\\\"\\\"", "0123456789", "tab\\t"];

random_string :: (builder: *String_Builder) {
    append(builder, "\"");

    for 1..pick(14) {
        if pick(8) == 0 {
            for 1..pick(40) append(builder, PIECES[pick(PIECES.count)]);
        } else {
            append(builder, PIECES[pick(PIECES.count)]);
        }
    }

    append(builder, "\"");
}

random_space :: (builder: *String_Builder) {
    if pick(3) == 0 append(builder, string.["  ", "\n", "\t", " \r\n    ", "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n"][pick(5)]);
}

NUMBERS :: string.["0", "-0", "7", "-12", "3.5", "1e3", "-2.5E-7", "123456789012345678901234567890", "0.1", "1.7976931348623157e308", "4.9e-324"];

random_value :: (builder: *String_Builder, depth: int) {
    random_space(builder);

    kind := pick(10);
    if depth > 5 kind = 3 + pick(7);

    if kind == 0 {
        append(builder, "{");

        for i: 0..pick(5) - 1 {
            if i append(builder, ",");

            random_space(builder);
            random_string(builder);
            random_space(builder);
            append(builder, ":");
            random_value(builder, depth + 1);
        }

        random_space(builder);
        append(builder, "}");
    } else if kind <= 2 {
        append(builder, "[");

        for i: 0..pick(6) - 1 {
            if i append(builder, ",");

            random_value(builder, depth + 1);
        }

        random_space(builder);
        append(builder, "]");
    } else if kind <= 5 {
        random_string(builder);
    } else if kind <= 8 {
        append(builder, NUMBERS[pick(NUMBERS.count)]);
    } else {
        append(builder, string.["true", "false", "null"][pick(3)]);
    }

    random_space(builder);
}

random_target_document :: (builder: *String_Builder) {
    append(builder, "[");

    for i: 0..pick(4) {
        if i append(builder, ",");

        append(builder, "{\"name\":");
        random_string(builder);
        append(builder, ",\"id\":");
        append(builder, NUMBERS[pick(4)]);
        append(builder, ",\"score\":");
        append(builder, NUMBERS[pick(NUMBERS.count)]);
        append(builder, ",\"tags\":[");
        random_string(builder);
        append(builder, ",");
        random_string(builder);
        append(builder, "],\"ok\":true");

        if pick(3) == 0 {
            append(builder, ",\"extra\":");
            random_value(builder, 3);
        }

        append(builder, "}");
    }

    append(builder, "]");
}

DAMAGE :: string.["\"", "\\", "{", "}", "[", "]", ",", ":", "0", "-", "e", ".", " ", "\x00", "\xff", "\n", "\x01", "\xc3", "\xa9", "t", "n", "u"];

damage :: (text: string) -> string {
    if text.count == 0 return text;

    builder: String_Builder;
    at := pick(text.count);
    kind := pick(4);

    append(*builder, slice(text, 0, at));

    if kind == 0 {
        // delete a byte
        append(*builder, slice(text, at + 1, text.count - at - 1));
    } else if kind == 1 {
        // insert
        append(*builder, DAMAGE[pick(DAMAGE.count)]);
        append(*builder, slice(text, at, text.count - at));
    } else if kind == 2 {
        // replace
        append(*builder, DAMAGE[pick(DAMAGE.count)]);
        append(*builder, slice(text, at + 1, text.count - at - 1));
    } else {
        // cut short
    }

    return builder_to_string(*builder);
}

main :: () {
    for SUITE_CASES same(it.text, it.name);

    random_seed(20261012);

    documents := 600;

    #if OS == .WASM documents = 100;

    for 1..documents {
        builder: String_Builder;

        if pick(3) == 0 random_target_document(*builder);
        else random_value(*builder, 0);

        text := builder_to_string(*builder);

        before_stats := json_index_stats.unindexable;

        same(text, "valid");
        check(json_index_stats.unindexable == before_stats, tprint("a valid text was not indexed: %", text));

        for 1..3 {
            text = damage(text);
            same(text, "damaged");
        }

        reset_temporary_storage();
    }

    // Large documents: thousands of index blocks, with and without damage.
    for 1..20 {
        builder: String_Builder;

        append(*builder, "[");

        for i: 0..150 {
            if i append(*builder, ",");

            random_value(*builder, 2);
        }

        append(*builder, "]");

        text := builder_to_string(*builder);

        same(text, "large");
        same(damage(text), "large, damaged");
        same(damage(damage(text)), "large, damaged twice");
    }

    // Documents far longer than one index block, with strings crossing the block boundaries at
    // every offset.
    for offset: 0..130 {
        builder: String_Builder;

        append(*builder, "[");
        for 1..offset append(*builder, " ");

        append(*builder, "\"");
        for 1..(offset % 7) append(*builder, "\\\\");

        append(*builder, "\\\"tail\\\\\"");
        for 1..offset append(*builder, " ");

        append(*builder, ", \"é\", 12345]");

        same(builder_to_string(*builder), "boundary");
    }

    json_use_simd = true;
    stats := json_index_stats;

    print("% texts compared; indexed %, unindexable %, reparsed %\n", compared, stats.parsed, stats.unindexable, stats.reparsed);
    #if OS != .WASM check(stats.parsed > 500 && stats.reparsed > 500 && stats.unindexable > 20, "the index paths were exercised");
    assert(failures == 0);
}
