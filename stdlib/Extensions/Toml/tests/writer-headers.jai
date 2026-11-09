#import "Basic";
#import "Extensions/Toml";

// Nested tables are written under `[table]` headers and arrays of tables under `[[array]]`;
// a table inside a mixed array stays inline. An empty table is a bare header.
SOURCE :: "title = 'x'\nserver = { host = 'a', tls = { on = true }, tags = [] }\nempty = {}\nmixed = [{ a = 1 }, 2]\nitems = [{ n = 1, sub = { k = 2 }, list = [{ z = 3 }] }, {}]\n";

EXPECTED :: "title = \"x\"\nmixed = [{a = 1}, 2]\n\n[server]\nhost = \"a\"\ntags = []\n\n[server.tls]\non = true\n\n[empty]\n\n[[items]]\nn = 1\n\n[items.sub]\nk = 2\n\n[[items.list]]\nz = 3\n\n[[items]]\n";

main :: () {
    doc, error := parse_toml(SOURCE);

    assert(error.kind == .NONE, error.message);
    defer free_toml(*doc);

    text, write_error := write_toml(doc.root);

    assert(write_error.kind == .NONE, write_error.message);
    defer free(text);
    assert(text == EXPECTED, text);

    again, again_error := parse_toml(text);

    assert(again_error.kind == .NONE, again_error.message);
    defer free_toml(*again);

    second, second_error := write_toml(again.root);

    assert(second_error.kind == .NONE);
    defer free(second);
    assert(second == text, second);
    print("ok\n");
}
