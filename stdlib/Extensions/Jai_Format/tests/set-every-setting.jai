#import "Basic";
#import "Extensions/Jai_Format";

// Every `[format]` setting can be set by name, and setting one leaves the others alone.
main :: () {
    config: Format_Config;

    assignments := string.[
        "indent_width=2",
        "case_indent=3",
        "case_body_indent=5",
        "max_blank_lines=1",
        "max_width=120",
        "brace_style=\"preserve\"",
        "blank_lines_between_items=false",
        "blank_line_after_declarations=true",
        "blank_lines_around_blocks=true",
        "blank_line_between_cases=true",
        "blank_line_before_comments=true",
        "blank_line_after_guards=true",
        "blank_line_before_return=true",
        "format_doc_examples=false",
        "ignore=[\"vendor\"]",
    ];

    for assignments {
        ok, error := format_apply_setting(*config, it);
        assert(ok, error);
    }

    assert(config.indent_width == 2);
    assert(config.case_indent == 3);
    assert(config.case_body_indent == 5);
    assert(config.max_blank_lines == 1);
    assert(config.max_width == 120);
    assert(config.brace_style == .PRESERVE);
    assert(!config.blank_lines_between_items);
    assert(config.blank_line_after_declarations);
    assert(config.blank_lines_around_blocks);
    assert(config.blank_line_between_cases);
    assert(config.blank_line_before_comments);
    assert(config.blank_line_after_guards);
    assert(config.blank_line_before_return);
    assert(!config.format_doc_examples);
    assert(config.ignore.count == 1 && config.ignore[0] == "vendor");

    // A later setting does not disturb the earlier ones, and one that fails changes nothing.
    ok, error := format_apply_setting(*config, "indent_width=8");

    assert(ok, error);
    assert(config.indent_width == 8 && config.case_indent == 3 && config.max_width == 120);

    ok, error = format_apply_setting(*config, "max_width=5");

    assert(!ok);
    assert(config.max_width == 120);

    for config.ignore free(it);

    array_free(config.ignore);
    print("ok\n");
}
