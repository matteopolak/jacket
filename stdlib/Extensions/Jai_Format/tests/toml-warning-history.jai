// An unknown key in `[format]` or `[doc.format]` is an error that names its path and line, however it is written.
#import "Basic";
#import "Extensions/Jai_Format";

rejects :: (text: string, message: string) {
    _, ok, error := parse_config(text);

    assert(!ok && error == message, error);
}

main :: () {
    rejects("[format.foo.bar]\nx=1\n", "line 1: unknown setting `format.foo.bar`");
    rejects("[format.foo]\nx=1\n[format.foo.bar]\ny=2\n", "line 1: unknown setting `format.foo`");
    rejects("[doc.format.foo.bar]\nx=1\n[doc.format.foo]\ny=2\n", "line 1: unknown setting `doc.format.foo.bar`");
    rejects("format.unknown=1\ndoc.format.other=2\nformat.last=3\n", "line 1: unknown setting `format.unknown`");
    rejects("format={extra.deep={x=1},colour={z=2}}\n", "line 1: unknown setting `format.extra.deep`");
    rejects("[format.deep.long.path]\nx=1\n[format]\nindent-width=2\n", "line 1: unknown setting `format.deep.long.path`");

    // The first unknown key in file order is the one reported, with the nearest known key named.
    rejects("[format]\nindent-width = 2\nindent-widht = 3\n[doc.format]\ncolour = 1\n", "line 3: unknown setting `format.indent-widht`; did you mean `format.indent-width`?");
    rejects("[doc.format]\nmax-width = 80\nmax-widht = 3\n[format]\ncolour = 1\n", "line 3: unknown setting `doc.format.max-widht`; did you mean `doc.format.max-width`?");

    config, ok, error := parse_config("[format.extra]\n");

    assert(!ok, error);
    config, ok, error = parse_config("[format]\nindent-width=2\n[doc.format]\nmax-width=80\n[project]\nx=1\n");
    assert(ok && config.indent_width == 2 && config.doc_format.count == 1, error);
    assert(config.doc_format[0].key == "max-width" && config.doc_format[0].value == "80");
    print("ok\n");
}
