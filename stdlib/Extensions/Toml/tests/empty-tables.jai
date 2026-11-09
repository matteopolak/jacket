#import "Basic";
#import "Extensions/Toml";

// The DOM writer and the typed encoder agree: an empty table is a `[header]`, except inside an
// array, where it stays inline.
Inner :: struct {
    on: bool; @toml(default, skip_serializing_if=zero)
}

Doc :: struct {
    name: string;
    inner: Inner; @toml(default)
}

main :: () {
    doc, error := parse_toml("name = 'x'\ninner = {}\nlist = [{}, 1]\n");

    assert(error.kind == .NONE, error.message);
    defer free_toml(*doc);

    dom_text, dom_error := write_toml(doc.root);

    assert(dom_error.kind == .NONE, dom_error.message);
    defer free(dom_text);
    assert(dom_text == "name = \"x\"\nlist = [{}, 1]\n\n[inner]\n", dom_text);

    value: Doc;
    value.name = "x";

    typed_text, typed_error := to_toml(value);

    assert(typed_error.kind == .NONE, typed_error.message);
    defer free(typed_text);
    assert(typed_text == "name = \"x\"\n\n[inner]\n", typed_text);

    // An empty table keeps its place on a round trip.
    again, again_error := parse_toml(dom_text);

    assert(again_error.kind == .NONE, again_error.message);
    defer free_toml(*again);

    second, second_error := write_toml(again.root);

    assert(second_error.kind == .NONE);
    defer free(second);
    assert(second == dom_text, second);
    print("ok\n");
}
