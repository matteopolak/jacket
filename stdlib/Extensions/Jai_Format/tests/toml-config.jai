#import "Basic";
#import "Extensions/Jai_Format";

main :: () {
    forms := string.[
        "[format]\nindent_width = 2\nbrace_style = 'preserve'\n",
        "format.indent_width = 2\nformat.brace_style = 'preserve'\n",
        "format = { indent_width = 2, brace_style = 'preserve' }\n",
        "[\"format\"]\n'indent_width' = 2\nbrace_style = 'preserve'\n",
    ];
    for forms {
        config, ok, error := parse_config(it);
        assert(ok, error);
        assert(config.indent_width == 2 && config.brace_style == .PRESERVE);
    }

    config, ok, error := parse_config("[doc.format]\nmax_width = 80\nindent_width = 2\n[format]\nindent_width = 4\nignore = ['vendor', \"**/generated_*.jai\"]\nunknown = 123\n");

    assert(ok, error);
    assert(config.indent_width == 4 && config.ignore.count == 2);
    assert(config.doc_format.count == 2);
    assert(config.doc_format[0].key == "max_width" && config.doc_format[0].value == "80");
    assert(config.doc_format[1].key == "indent_width" && config.doc_format[1].value == "2");
    assert(config.unknown_keys.count == 1 && config.unknown_keys[0] == "unknown");
    set_ok, set_error := format_apply_setting(*config, "indent_width=8");

    assert(set_ok && config.indent_width == 8, set_error);
    doc_ok, doc_error := format_apply_setting(*config, "doc.format.indent_width=3");

    assert(doc_ok && config.doc_format[2].value == "3", doc_error);

    // Intentional migration: malformed unrelated syntax and duplicate settings now fail.
    _, malformed_ok := parse_config("[unrelated]\nnot a TOML assignment\n[format]\nindent_width = 2\n");

    assert(!malformed_ok);
    _, repeated_ok := parse_config("[format]\nindent_width = 2\nindent_width = 3\n");

    assert(!repeated_ok);
    rich, rich_ok, rich_error := parse_config("[format]\nindent_width = 0x2\nbrace_style = \"pres\\u0065rve\"\nignore = [\"vendor\\u002f*\"]\n[unrelated]\nwhen = 2000-01-01T12:34Z\nvalues = {a = 1,}\n");

    assert(rich_ok && rich.indent_width == 2 && rich.brace_style == .PRESERVE, rich_error);
    assert(rich.ignore[0] == "vendor/*");
    doc_rich, doc_rich_ok := parse_config("[doc.format]\nindent_width = 0x2\nbrace_style = \"pres\\u0065rve\"\n");

    assert(doc_rich_ok && doc_rich.doc_format[0].value == "0x2" && doc_rich.doc_format[1].value == "\"pres\\u0065rve\"");
    print("ok\n");
}
