// Documentation declarations have a separate canonical layout; ordinary format_source keeps
// its existing source-preserving policy. Run with jaic declarations.jai -interp.
#import "Basic";
#import "String";
#import "Extensions/Jai_Format";

SOURCE :: #string JAI
Box     ::      struct     {

       first      :       int;


       second     :       []    string;
}
JAI

EXPECTED :: #string JAI
Box :: struct {
    first: int;
    second: [] string;
}
JAI

main :: () {
    formatted, ok, error := format_declaration(SOURCE);

    assert(ok, error);
    assert(formatted == EXPECTED, formatted);
    again, again_ok, again_error := format_declaration(formatted);

    assert(again_ok && again == formatted, again_error);
    equivalent, why := format_tokens_equivalent(SOURCE, formatted);

    assert(equivalent, why);
    free(again);
    free(formatted);
    joined, joined_ok, joined_error := format_declaration("Short::struct {\n value:\n                  *Entry;\n}\n");

    assert(joined_ok, joined_error);
    assert(joined == "Short :: struct {\n    value: *Entry;\n}\n", joined);
    free(joined);

    narrow: Format_Config;
    narrow.max_width = 44;
    wide_source := "Callbacks::struct {callback:(first:*LongParameterType,second:*AnotherParameterType)->*Return_Type;}\n";
    wrapped, wrapped_ok, wrapped_error := format_declaration(wide_source, narrow);

    assert(wrapped_ok, wrapped_error);
    assert(format_count_long_lines(wrapped, narrow.max_width) == 0, wrapped);
    stable, stable_ok, stable_error := format_declaration(wrapped, narrow);

    assert(stable_ok && stable == wrapped, stable_error);
    unchanged, unchanged_ok := format_source(wide_source, narrow);

    assert(unchanged_ok && format_count_long_lines(unchanged, narrow.max_width) > 0, "ordinary source wrapping changed");
    free(stable);
    free(unchanged);
    free(wrapped);

    comment_source := "Commented::struct{\n// hard break  \n// next\n// ```jai\n// value := \"a  b\";  \n// \tkeep  \t\n// ```\nvalue:int; // trailing  \n}\n";
    comments, comments_ok, comments_error := format_declaration(comment_source);

    assert(comments_ok, comments_error);
    assert(contains(comments, "// hard break  \n"), comments);
    assert(contains(comments, "// value := \"a  b\";  \n"), comments);
    assert(contains(comments, "// \tkeep  \t\n"), comments);
    assert(contains(comments, "// trailing  \n"), comments);
    free(comments);

    prose_source := "Docs::struct{\n// This paragraph documents the entire field and keeps `two  spaces` intact while prose wraps deterministically.\n// A second source line joins the same paragraph.\n//\n// This is another paragraph.\nvalue:int;\n}\n";
    prose, prose_ok, prose_error := format_declaration(prose_source, narrow);

    assert(prose_ok, prose_error);
    prose_expected := "Docs :: struct {\n    // This paragraph documents the entire\n    // field and keeps `two  spaces` intact\n    // while prose wraps deterministically.\n    // A second source line joins the same\n    // paragraph.\n    //\n    // This is another paragraph.\n    value: int;\n}\n";

    assert(prose == prose_expected, prose);
    prose_again, prose_again_ok, prose_again_error := format_declaration(prose, narrow);

    assert(prose_again_ok && prose_again == prose, prose_again_error);
    assert(format_count_long_lines(prose, narrow.max_width) == 0);
    free(prose_again);
    free(prose);

    spacious := "Spacious::struct{\n\n      first     : [] string = \"keep  spaces\";\n\n      left  , right    :int;\n}\n";
    compact, compact_ok, compact_error := format_declaration(spacious);

    assert(compact_ok, compact_error);
    assert(compact == "Spacious :: struct {\n    first: [] string = \"keep  spaces\";\n    left, right: int;\n}\n", compact);
    free(compact);

    inline_source := "Inline::struct{\n// This is a paragraph with many words `inline` remaining.\nvalue:int;\n}\n";
    inline_text, inline_ok, inline_error := format_declaration(inline_source, narrow);

    assert(inline_ok, inline_error);
    assert(inline_text == "Inline :: struct {\n    // This is a paragraph with many words\n    // `inline` remaining.\n    value: int;\n}\n", inline_text);
    free(inline_text);

    _, invalid_ok, invalid_error := format_declaration("Broken :: struct { field: (int; }\n");

    assert(!invalid_ok && invalid_error.count > 0);
    print("ok\n");
}
