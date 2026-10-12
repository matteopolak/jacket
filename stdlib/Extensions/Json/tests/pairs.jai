// from_pairs and from_text: typed values from query strings, forms and single strings, with the
// notes and the error messages of from_json.
#import "Basic";
#import "Extensions/Json";

Color :: enum { red; green; blue; }

Search :: struct @json(rename_all=camelCase) {
    q: string;
    page_size := 20;     @json(default)
    tags: [..] string;   @json(default)
    ratio: float64;      @json(default)
    exact: bool;         @json(default)
    color := Color.green; @json(default,alias=colour)
    ids: [..] int;       @json(default)
}

Strict :: struct @json(deny_unknown_fields) {
    name: string;
}

pairs :: (items: ..string) -> [] Text_Pair {
    result: [..] Text_Pair;

    for 0..items.count / 2 - 1 array_add(*result, .{ items[it * 2], items[it * 2 + 1] });

    return result;
}

fail :: (message: string) {
    print("FAIL: %\n", message);
    exit(1);
}

expect :: (ok: bool, message: string) {
    if !ok fail(message);
}

main :: () {
    search, error := from_pairs(Search, pairs("q", "jai", "pageSize", "50", "tags", "a", "tags", "b", "ratio", "0.5", "exact", "true", "colour", "blue", "ids", "3", "ids", "4", "other", "ignored"));

    expect(error.kind == .NONE, tprint("pairs: %", json_error_text(error)));
    expect(search.q == "jai" && search.page_size == 50, "strings and numbers");
    expect(search.tags.count == 2 && search.tags[1] == "b", "repeated keys make an array");
    expect(search.ratio == 0.5 && search.exact && search.color == .blue, "float, bool, alias and enum");
    expect(search.ids.count == 2 && search.ids[0] == 3 && search.ids[1] == 4, "numeric array");
    free_json(*search);

    // Defaults and missing fields.
    only_q, only_error := from_pairs(Search, pairs("q", "x"));
    expect(only_error.kind == .NONE && only_q.page_size == 20 && only_q.color == .green && only_q.tags.count == 0, "defaults");
    free_json(*only_q);

    nothing, missing := from_pairs(Search, pairs());
    expect(missing.kind == .MISSING_FIELD, tprint("missing q: %", json_error_text(missing)));
    free_json_error(*missing);

    // Wrong types name the field.
    bad, bad_error := from_pairs(Search, pairs("q", "x", "pageSize", "many"));
    expect(bad_error.kind == .TYPE && bad_error.path == "pageSize", tprint("bad int: %", json_error_text(bad_error)));
    free_json_error(*bad_error);

    bad, bad_error = from_pairs(Search, pairs("q", "x", "pageSize", "1.5"));
    expect(bad_error.kind == .TYPE && bad_error.path == "pageSize", "a fraction is not an integer");
    free_json_error(*bad_error);

    bad, bad_error = from_pairs(Search, pairs("q", "x", "color", "purple"));
    expect(bad_error.kind == .INVALID_VALUE, tprint("bad enum: %", json_error_text(bad_error)));
    free_json_error(*bad_error);

    bad, bad_error = from_pairs(Search, pairs("q", "x", "q", "y"));
    expect(bad_error.kind == .DUPLICATE_FIELD, tprint("duplicate: %", json_error_text(bad_error)));
    free_json_error(*bad_error);

    bad, bad_error = from_pairs(Search, pairs("q", "x", "exact", "yes"));
    expect(bad_error.kind == .TYPE && bad_error.path == "exact", "bool spelling");
    free_json_error(*bad_error);

    // Unknown keys.
    strict, strict_error := from_pairs(Strict, pairs("name", "a", "extra", "b"));
    expect(strict_error.kind == .UNKNOWN_FIELD, tprint("deny_unknown_fields: %", json_error_text(strict_error)));
    free_json_error(*strict_error);

    // Text that needs escaping survives.
    quoted, quoted_error := from_pairs(Strict, pairs("name", "say \"hi\"\n\\"));
    expect(quoted_error.kind == .NONE && quoted.name == "say \"hi\"\n\\", "escapes");
    free_json(*quoted);

    // from_text.
    number, number_error := from_text(int, "42");
    expect(number_error.kind == .NONE && number == 42, "from_text int");

    negative, negative_error := from_text(s8, "-128");
    expect(negative_error.kind == .NONE && negative == -128, "from_text s8");

    _, range_error := from_text(u8, "300");
    expect(range_error.kind == .OUT_OF_RANGE, tprint("from_text range: %", json_error_text(range_error)));
    free_json_error(*range_error);

    _, text_error := from_text(int, "abc");
    expect(text_error.kind == .TYPE, "from_text type");
    free_json_error(*text_error);

    _, empty_error := from_text(int, "");
    expect(empty_error.kind == .TYPE, "empty is not a number");
    free_json_error(*empty_error);

    ratio, ratio_error := from_text(float64, "1e3");
    expect(ratio_error.kind == .NONE && ratio == 1000, "from_text float");

    flag, flag_error := from_text(bool, "true");
    expect(flag_error.kind == .NONE && flag, "from_text bool");

    color, color_error := from_text(Color, "blue");
    expect(color_error.kind == .NONE && color == .blue, "from_text enum");

    word, word_error := from_text(string, "he said \"x\"");
    expect(word_error.kind == .NONE && word == "he said \"x\"", "from_text string");
    free_json(*word);

    print("ok\n");
}
