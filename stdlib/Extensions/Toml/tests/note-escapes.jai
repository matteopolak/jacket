#import "Basic";
#import "Extensions/Toml";

// Escapes in a quoted note value are resolved: `\t` is a tab and `\"` a quote, both ways.
Doc :: struct {
    value: string; @toml(rename="say \"hi\"\tthen")
}

main :: () {
    doc: Doc;
    doc.value = "x";
    text, error := to_toml(doc);

    assert(error.kind == .NONE, error.message);
    defer free(text);
    assert(text == "\"say \\\"hi\\\"\\u0009then\" = \"x\"\n", text);
}
