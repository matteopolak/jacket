#import "Basic";
#import "Extensions/Jai_Format";

main :: () {
    nested, nested_ok := parse_config("[format.foo.bar]\nx=1\n");

    assert(nested_ok && nested.unknown_keys.count == 1 && nested.unknown_keys[0] == "format.foo.bar");
    two, two_ok := parse_config("[format.foo]\nx=1\n[format.foo.bar]\ny=2\n");

    assert(two_ok && two.unknown_keys.count == 2 && two.unknown_keys[0] == "format.foo" && two.unknown_keys[1] == "format.foo.bar");
    docs, docs_ok := parse_config("[doc.format.foo.bar]\nx=1\n[doc.format.foo]\ny=2\n");

    assert(docs_ok && docs.unknown_keys.count == 2 && docs.unknown_keys[0] == "doc.format.foo.bar" && docs.unknown_keys[1] == "doc.format.foo");
    dotted, dotted_ok := parse_config("format.unknown=1\ndoc.format.other=2\nformat.last=3\n");

    assert(dotted_ok && dotted.unknown_keys.count == 3 && dotted.unknown_keys[0] == "unknown" && dotted.unknown_keys[1] == "doc.format.other" && dotted.unknown_keys[2] == "last");
    inline_config, inline_ok := parse_config("format={extra.deep={x=1},colour={z=2}}\n");

    assert(inline_ok && inline_config.unknown_keys.count == 2 && inline_config.unknown_keys[0] == "extra.deep" && inline_config.unknown_keys[1] == "colour");
    mixed, mixed_ok := parse_config("[format.deep.long.path]\nx=1\n[format]\nindent_width=2\n[doc.format.deep.more]\nx=1\n[doc.format]\nmax_width=80\n");

    assert(mixed_ok && mixed.indent_width == 2);
    assert(mixed.unknown_keys.count == 2 && mixed.unknown_keys[0] == "format.deep.long.path" && mixed.unknown_keys[1] == "doc.format.deep.more");
    assert(mixed.doc_format.count == 1 && mixed.doc_format[0].key == "max_width" && mixed.doc_format[0].value == "80");
    print("ok\n");
}
