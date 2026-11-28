#import "Basic";
#import "Extensions/Jai_Format";

// Every `[format]` setting can be set by name, and setting one leaves the others alone.
main :: () {
    config: Format_Config;

    assignments := string.[
        "indent-width=2",
        "case-indent=3",
        "case-body-indent=5",
        "max-blank-lines=1",
        "max-width=120",
        "brace-style=\"preserve\"",
        "blank-lines-between-items=false",
        "blank-line-after-declarations=true",
        "blank-lines-around-blocks=true",
        "blank-line-between-cases=true",
        "blank-line-before-comments=true",
        "blank-line-after-guards=true",
        "blank-line-before-return=true",
        "format-doc-examples=false",
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
    ok, error := format_apply_setting(*config, "indent-width=8");

    assert(ok, error);
    assert(config.indent_width == 8 && config.case_indent == 3 && config.max_width == 120);

    ok, error = format_apply_setting(*config, "max-width=5");

    assert(!ok);
    assert(config.max_width == 120);

    for config.ignore free(it);

    array_free(config.ignore);
    print("ok\n");
}
