// from_query and from_param: typed values from query strings, forms and single strings, with the
// notes and the error messages of from_json.
#import "Basic";
#import "Extensions/Json";

Color :: enum {
    red;
    green;
    blue;
}

Search :: struct @json(rename_all=camelCase) {
    q: string;
    page_size := 20; @json(default)
    tags: [..] string; @json(default)
    ratio: float64; @json(default)
    exact: bool; @json(default)
    color := Color.green; @json(default,alias=colour)
    ids: [..] int; @json(default)
}

Strict :: struct @json(deny_unknown_fields) {
    name: string;
}

pairs :: (items: ..string) -> [] Query_Pair {
    result: [..] Query_Pair;

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
    search, error := from_query(Search, pairs("q", "jai", "pageSize", "50", "tags", "a", "tags", "b", "ratio", "0.5", "exact", "true", "colour", "blue", "ids", "3", "ids", "4", "other", "ignored"));

    expect(error.kind == .NONE, tprint("pairs: %", error_text(error)));
    expect(search.q == "jai" && search.page_size == 50, "strings and numbers");
    expect(search.tags.count == 2 && search.tags[1] == "b", "repeated keys make an array");
    expect(search.ratio == 0.5 && search.exact && search.color == .blue, "float, bool, alias and enum");
    expect(search.ids.count == 2 && search.ids[0] == 3 && search.ids[1] == 4, "numeric array");
    free_json(*search);

    // Defaults and missing fields.
    only_q, only_error := from_query(Search, pairs("q", "x"));

    expect(only_error.kind == .NONE && only_q.page_size == 20 && only_q.color == .green && only_q.tags.count == 0, "defaults");
    free_json(*only_q);

    _, missing := from_query(Search, pairs());

    expect(missing.kind == .MISSING_FIELD, tprint("missing q: %", error_text(missing)));
    free_json(*missing);

    // Wrong types name the field.
    bad, bad_error := from_query(Search, pairs("q", "x", "pageSize", "many"));

    expect(bad_error.kind == .TYPE && bad_error.path == "pageSize", tprint("bad int: %", error_text(bad_error)));
    free_json(*bad_error);

    bad, bad_error = from_query(Search, pairs("q", "x", "pageSize", "1.5"));
    expect(bad_error.kind == .TYPE && bad_error.path == "pageSize", "a fraction is not an integer");
    free_json(*bad_error);

    bad, bad_error = from_query(Search, pairs("q", "x", "color", "purple"));
    expect(bad_error.kind == .INVALID_VALUE, tprint("bad enum: %", error_text(bad_error)));
    free_json(*bad_error);

    bad, bad_error = from_query(Search, pairs("q", "x", "q", "y"));
    expect(bad_error.kind == .DUPLICATE_FIELD, tprint("duplicate: %", error_text(bad_error)));
    free_json(*bad_error);

    bad, bad_error = from_query(Search, pairs("q", "x", "exact", "yes"));
    expect(bad_error.kind == .TYPE && bad_error.path == "exact", "bool spelling");
    free_json(*bad_error);

    // Unknown keys.
    _, strict_error := from_query(Strict, pairs("name", "a", "extra", "b"));

    expect(strict_error.kind == .UNKNOWN_FIELD, tprint("deny_unknown_fields: %", error_text(strict_error)));
    free_json(*strict_error);

    // Text that needs escaping survives.
    quoted, quoted_error := from_query(Strict, pairs("name", "say \"hi\"\n\\"));

    expect(quoted_error.kind == .NONE && quoted.name == "say \"hi\"\n\\", "escapes");
    free_json(*quoted);

    // from_param.
    number, number_error := from_param(int, "42");

    expect(number_error.kind == .NONE && number == 42, "from_param int");

    negative, negative_error := from_param(s8, "-128");

    expect(negative_error.kind == .NONE && negative == -128, "from_param s8");

    _, range_error := from_param(u8, "300");

    expect(range_error.kind == .OUT_OF_RANGE, tprint("from_param range: %", error_text(range_error)));
    free_json(*range_error);

    _, text_error := from_param(int, "abc");

    expect(text_error.kind == .TYPE, "from_param type");
    free_json(*text_error);

    _, empty_error := from_param(int, "");

    expect(empty_error.kind == .TYPE, "empty is not a number");
    free_json(*empty_error);

    ratio, ratio_error := from_param(float64, "1e3");

    expect(ratio_error.kind == .NONE && ratio == 1000, "from_param float");

    flag, flag_error := from_param(bool, "true");

    expect(flag_error.kind == .NONE && flag, "from_param bool");

    color, color_error := from_param(Color, "blue");

    expect(color_error.kind == .NONE && color == .blue, "from_param enum");

    word, word_error := from_param(string, "he said \"x\"");

    expect(word_error.kind == .NONE && word == "he said \"x\"", "from_param string");
    free_json(*word);

    print("ok\n");
}
