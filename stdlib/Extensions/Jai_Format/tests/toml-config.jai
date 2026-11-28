#import "Basic";
#import "String";
#import "Extensions/Jai_Format";

main :: () {
    forms := string.[
        "[format]\nindent-width = 2\nbrace-style = 'preserve'\n",
        "format.indent-width = 2\nformat.brace-style = 'preserve'\n",
        "format = { indent-width = 2, brace-style = 'preserve' }\n",
        "[\"format\"]\n'indent-width' = 2\nbrace-style = 'preserve'\n",
    ];
    for forms {
        config, ok, error := parse_config(it);
        assert(ok, error);
        assert(config.indent_width == 2 && config.brace_style == .PRESERVE);
    }

    config, ok, error := parse_config("[doc.format]\nmax-width = 80\nindent-width = 2\n[format]\nindent-width = 4\nignore = ['vendor', \"**/generated_*.jai\"]\n");

    assert(ok, error);
    assert(config.indent_width == 4 && config.ignore.count == 2);
    assert(config.doc_format.count == 2);
    assert(config.doc_format[0].key == "max-width" && config.doc_format[0].value == "80");
    assert(config.doc_format[1].key == "indent-width" && config.doc_format[1].value == "2");
    set_ok, set_error := format_apply_setting(*config, "indent-width=8");

    assert(set_ok && config.indent_width == 8, set_error);
    doc_ok, doc_error := format_apply_setting(*config, "doc.format.indent-width=3");

    assert(doc_ok && config.doc_format[2].value == "3", doc_error);

    // A bad value is reported without the line prefix a file would give it, and nothing is stored.
    bad_doc_ok, bad_doc_error := format_apply_setting(*config, "doc.format.indent-width=x");

    assert(!bad_doc_ok && bad_doc_error.count && !begins_with(bad_doc_error, "line "), bad_doc_error);
    assert(config.doc_format.count == 3);
    bad_ok, bad_error := format_apply_setting(*config, "indent-width=x");

    assert(!bad_ok && bad_error.count && !begins_with(bad_error, "line "), bad_error);
    assert(config.indent_width == 8);
    unknown_ok, unknown_error := format_apply_setting(*config, "doc.format.colour=1");

    assert(!unknown_ok && unknown_error == "unknown setting `doc.format.colour`", unknown_error);

    // Unknown keys name the nearest known one only when it is close; a key too long to compare names none.
    _, near_ok, near_error := parse_config("[format]\nindent-widht = 4\n");

    assert(!near_ok && near_error == "line 2: unknown setting `format.indent-widht`; did you mean `format.indent-width`?", near_error);
    long_key := "this_setting_name_is_far_longer_than_any_known_one_and_is_past_sixty_three_bytes";
    _, far_ok, far_error := parse_config(tprint("[format]\n% = 4\n", long_key));

    assert(!far_ok && far_error == tprint("line 2: unknown setting `format.%`", long_key), far_error);

    // Intentional migration: malformed unrelated syntax and duplicate settings now fail.
    _, malformed_ok := parse_config("[unrelated]\nnot a TOML assignment\n[format]\nindent-width = 2\n");

    assert(!malformed_ok);
    _, repeated_ok := parse_config("[format]\nindent-width = 2\nindent-width = 3\n");

    assert(!repeated_ok);
    rich, rich_ok, rich_error := parse_config("[format]\nindent-width = 0x2\nbrace-style = \"pres\\u0065rve\"\nignore = [\"vendor\\u002f*\"]\n[unrelated]\nwhen = 2000-01-01T12:34Z\nvalues = {a = 1,}\n");

    assert(rich_ok && rich.indent_width == 2 && rich.brace_style == .PRESERVE, rich_error);
    assert(rich.ignore[0] == "vendor/*");
    doc_rich, doc_rich_ok := parse_config("[doc.format]\nindent-width = 0x2\nbrace-style = \"pres\\u0065rve\"\n");

    assert(doc_rich_ok && doc_rich.doc_format[0].value == "0x2" && doc_rich.doc_format[1].value == "\"pres\\u0065rve\"");
    print("ok\n");
}
