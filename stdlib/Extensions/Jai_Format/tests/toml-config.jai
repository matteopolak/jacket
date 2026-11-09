#import "Basic";
#import "String";
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

    // A bad value is reported without the line prefix a file would give it, and nothing is stored.
    bad_doc_ok, bad_doc_error := format_apply_setting(*config, "doc.format.indent_width=x");

    assert(!bad_doc_ok && bad_doc_error.count && !begins_with(bad_doc_error, "line "), bad_doc_error);
    assert(config.doc_format.count == 3);
    bad_ok, bad_error := format_apply_setting(*config, "indent_width=x");

    assert(!bad_ok && bad_error.count && !begins_with(bad_error, "line "), bad_error);
    assert(config.indent_width == 8);
    unknown_ok, unknown_error := format_apply_setting(*config, "doc.format.colour=1");

    assert(!unknown_ok && unknown_error == "unknown setting `doc.format.colour`", unknown_error);

    // Unknown keys name the nearest known one only when it is close; a key too long to compare names none.
    assert(format_unknown_setting_note("format.indent_widht", "a.toml") == "unknown setting `format.indent_widht` in a.toml (ignored); did you mean `format.indent_width`?");
    assert(format_unknown_setting_note("indent_widht", "a.toml") == "unknown setting `format.indent_widht` in a.toml (ignored); did you mean `format.indent_width`?");
    long_key := "format.this_setting_name_is_far_longer_than_any_known_one_and_is_past_sixty_three_bytes";

    assert(format_unknown_setting_note(long_key, "a.toml") == tprint("unknown setting `%` in a.toml (ignored)", long_key));

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
