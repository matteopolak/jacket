// JSON parsing and writing with rluba/jaison, generic and typed, over a generated document.
// Copied into the project as its entry.
#import "Basic";
#import "String";
#load "module.jai";

Item :: struct {
    id: int;
    name: string;
    score: float64;
    active: bool;
    tags: [..] string;
}

main :: () {
    builder: String_Builder;

    append(*builder, "[");

    for 0..60_000 - 1 {
        if it append(*builder, ",");
        print_to_builder(*builder, "{\"id\": %, \"name\": \"item-%\", \"score\": %, \"active\": %, \"tags\": [\"a%\", \"b%\", \"c\"]}",
                         it, it * 7, it % 1000, ifx it % 3 == 0 then "true" else "false", it % 17, it % 5);
    }

    append(*builder, "]");
    text := builder_to_string(*builder);

    for round: 0..2 {
        ok, items := json_parse_string(text, [] Item);

        assert(ok);
        total := 0;
        for items total += it.id + it.tags.count + ifx it.active then 1 else 0;

        generic_ok, value := json_parse_string(text);

        assert(generic_ok);

        written := json_write_string(items, indent_char = "");

        print("% % % %\n", items.count, total, value.type, written.count);
    }
}
