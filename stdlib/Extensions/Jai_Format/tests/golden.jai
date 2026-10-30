// test: skip_on(.WASM, reason = "File_Utilities has no wasm backend")
// Jai_Format golden tests: every fixtures/<name>.in.jai formats to <name>.out.jai (with
// <name>.toml as its config when present), formatting is idempotent and keeps the token stream,
// malformed input is refused, and the formatter's own sources are already formatted.
//
//     jaic stdlib/Extensions/Jai_Format/tests/golden.jai -interp              check
//     jaic stdlib/Extensions/Jai_Format/tests/golden.jai -interp -- --bless   rewrite the .out.jai files

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Sort";

#import "Extensions/Jai_Format";

CASES :: "fixtures";
SOURCES :: string.["../module.jai", "../lexer.jai", "../config.jai", "../format.jai", "golden.jai", "../../../../jaifmt/main.jai"];

failures := 0;

fail :: (format: string, args: ..Any) {
    print("FAIL: ");
    print(format, ..args);
    print("\n");
    failures += 1;
}

main :: () {
    args := get_command_line_arguments();
    bless := args.count > 1 && args[1] == "--bless";

    names: [..] string;

    for file_list(CASES) {
        if ends_with(it, ".in.jai") array_add(*names, slice(it, 0, it.count - ".in.jai".count));
    }

    quick_sort(names, compare_strings);
    if names.count < 8 fail("expected the golden cases in %, found %", CASES, names.count);

    for base: names {
        config: Format_Config;
        toml, has_config := read_entire_file(tprint("%.toml", base), log_errors = false);

        if has_config {
            parsed, ok, message := parse_config(toml);
            if !ok {
                fail("%.toml: %", base, message);
                continue;
            }

            config = parsed;
        }

        input, read_ok := read_entire_file(tprint("%.in.jai", base));
        if !read_ok {
            fail("cannot read %.in.jai", base);
            continue;
        }

        output, ok, error := format_source(input, config);
        if !ok {
            fail("%.in.jai: %", base, error);
            continue;
        }

        again, again_ok := format_source(output, config);
        if !again_ok || again != output fail("%: formatting is not idempotent", base);

        expected_path := tprint("%.out.jai", base);

        if bless {
            write_entire_file(expected_path, output.data, output.count);
            continue;
        }

        expected, expected_ok := read_entire_file(expected_path, log_errors = false);
        if !expected_ok {
            fail("missing %; run with -- --bless", expected_path);
            continue;
        }

        if expected != output report_difference(base, expected, output);
    }

    check_refusals();
    check_equivalence_guard();
    check_config_parsing();
    check_toml_forms();
    check_self_formatted();

    if failures {
        print("% jaifmt golden failures\n", failures);
        exit(1);
    }

    print("jaifmt golden tests: % cases ok\n", names.count);
}

report_difference :: (name: string, expected: string, actual: string) {
    a := split(expected, "\n");
    b := split(actual, "\n");

    for i: 0..max(a.count, b.count) - 1 {
        line_a := ifx i < a.count then a[i] else "<end>";
        line_b := ifx i < b.count then b[i] else "<end>";

        if line_a != line_b {
            fail("%.out.jai line %:\n  expected: %\n  actual:   %", name, i + 1, line_a, line_b);
            return;
        }
    }

    fail("%.out.jai differs", name);
}

// Input that cannot be formatted safely is refused, not mangled.
check_refusals :: () {
    bad :: string.[
        "f :: () { x := 1;\n", // unclosed block
        "f :: () { x := (1; }\n", // mismatched bracket
        "s := \"unterminated;\n",
        "t := #string END\nno terminator\n",
        "/* unterminated comment\n",
        "f :: () {} @Note(open\n\n", // a note's open `(` runs to the end of the file
        "x := 1 \x01 2;\n", // a byte the lexer does not know
    ];
    for bad {
        _, ok := format_source(it);
        if ok fail("malformed input was formatted: %", it);
    }
}

// The guard itself: different token streams, line breaks or comments are reported.
check_equivalence_guard :: () {
    same, why := format_tokens_equivalent("a := b+c; // x\n", "a := b + c;  // x\n");
    if !same fail("equivalent sources rejected: %", why);

    pairs :: string.[
        "x := a & &b;\n", "x := a &&b;\n",
        "a := 1;\n", "a := 2;\n",
        "x := a // note\n;\n", "x := a;\n",
        // After `}`, a line starting with `-` is a new statement; on the same line it subtracts.
        "f :: () {}\n-b;\n", "f :: () {} -b;\n",
        "x := a;\n@Note b;\n", "x := a; @Note b;\n",
        "#if A x := 1\n#align 8;\n", "#if A x := 1 #align 8;\n",
        "s := \"a b\";\n", "s := \"a  b\";\n",
        // Directive flags are only flags when written against the directive.
        "k :: #library,system \"k\";\n", "k :: #library, system \"k\";\n",
        "k :: #library ,system \"k\";\n", "k :: #library,system \"k\";\n",
        "l := #location(x);\n", "l := #location (x);\n",
    ];
    i := 0;

    while i < pairs.count {
        equivalent := format_tokens_equivalent(pairs[i], pairs[i + 1]);
        if equivalent fail("format_tokens_equivalent accepted % -> %", pairs[i], pairs[i + 1]);
        i += 2;
    }
}

check_config_parsing :: () {
    config, ok, message := parse_config("[format]\nindent_width = 2 # two\nmax_width = 120\nbrace_style = \"preserve\"\nignore = [\"a/**\", \"b.jai\"]\n");
    if !ok fail("config rejected: %", message);

    if config.indent_width != 2 || config.max_width != 120 || config.brace_style != .PRESERVE || config.ignore.count != 2 {
        fail("config parsed wrong: %", config);
    }

    if !config.blank_lines_between_items fail("blank_lines_between_items should default to true");

    off, off_ok := parse_config("[format]\nblank_lines_between_items = false\n");
    if !off_ok || off.blank_lines_between_items fail("blank_lines_between_items = false not parsed");

    if config.blank_line_after_declarations || config.blank_lines_around_blocks || config.blank_line_between_cases || config.blank_line_before_comments || config.blank_line_after_guards || config.blank_line_before_return {
        fail("the code block blank line options should default to false");
    }

    on, on_ok := parse_config("[format]\nblank_line_after_declarations = true\nblank_lines_around_blocks = true\nblank_line_between_cases = true\nblank_line_before_comments = true\nblank_line_after_guards = true\nblank_line_before_return = true\n");
    if !on_ok || !on.blank_line_after_declarations || !on.blank_lines_around_blocks || !on.blank_line_between_cases || !on.blank_line_before_comments || !on.blank_line_after_guards || !on.blank_line_before_return {
        fail("the code block blank line options were not parsed");
    }

    for string.["[format]\nindent_width = x\n", "[format]\nbrace_style = \"weird\"\n", "[format]\nignore = \"a\"\n", "[format]\nblank_lines_between_items = 1\n", "[format]\nblank_lines_around_blocks = yes\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad config accepted: %", it);
    }

    jacket, jacket_ok, jacket_message := parse_config("[project]\nentry = [\n  \"a.jai\", # the program\n]\n\n[format]\nindent_width = 2 # two\nignore = [\n  \"a/**\", # first\n  \"b.jai\",\n]\nbrace_style = \"preserve\"\n\n[lint.rules]\nindent_width = \"deny\"\n");
    if !jacket_ok fail("jacket.toml rejected: %", jacket_message);

    if jacket.indent_width != 2 || jacket.brace_style != .PRESERVE || jacket.ignore.count != 2 || jacket.ignore[1] != "b.jai" {
        fail("jacket.toml [format] parsed wrong: %", jacket);
    }

    none, none_ok := parse_config("[lint]\nmax_nesting = 3\n");
    if !none_ok || none.indent_width != 4 fail("a jacket.toml without [format] should give the defaults");

    for string.["[format]\nindent_width = x\n", "[format]\nignore = [\"a\"\n", "[format]\nindent_width\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad jacket.toml accepted: %", it);
    }

    unknown, unknown_ok := parse_config("[format]\nindent_widht = 2\ncolour = 1\n[format.extra]\nx = 1\n");
    if !unknown_ok || unknown.unknown_keys.count != 3 || unknown.indent_width != 4 fail("unknown [format] keys should be dropped and listed");

    note := format_unknown_setting_note(unknown.unknown_keys[0], "jacket.toml");
    if note != "unknown setting `format.indent_widht` in jacket.toml (ignored); did you mean `format.indent_width`?" fail("unknown key note: %", note);

    note = format_unknown_setting_note(unknown.unknown_keys[1], "jacket.toml");
    if note != "unknown setting `format.colour` in jacket.toml (ignored)" fail("unknown key note: %", note);

    config.root = "/repo";
    if !format_is_ignored(config, "/repo/a/x/y.jai") fail("`a/**` should ignore /repo/a/x/y.jai");
    if !format_is_ignored(config, "/repo/b.jai") fail("`b.jai` should ignore /repo/b.jai");
    if format_is_ignored(config, "/repo/c/b.jai") fail("`b.jai` should not ignore /repo/c/b.jai");
    if !format_glob_match("**/gen_*.jai", "x/y/gen_a.jai") || !format_glob_match("**/gen_*.jai", "gen_b.jai") fail("`**/` glob");
    if format_glob_match("*.jai", "dir/a.jai") fail("`*` must not cross directories");

    check_shared_globs();
    check_applied_settings();
}

// Settings handed over as `key=value` (what `jacket fmt` does) read like the file's values.
check_applied_settings :: () {
    applied: Format_Config;

    for string.["indent_width=2", "brace_style=\"preserve\"", "blank_line_before_return = true", "max_width=80"] {
        ok, message := format_apply_setting(*applied, it);
        if !ok fail("--set %: %", it, message);
    }

    if applied.indent_width != 2 || applied.brace_style != .PRESERVE || !applied.blank_line_before_return || applied.max_width != 80 fail("applied settings wrong: %", applied);

    for string.["indent_width=0", "indent_width=x", "colour=1", "indent_width", "brace_style=same_line", "blank_lines_between_items=1"] {
        ok, message := format_apply_setting(*applied, it);
        if ok fail("--set % should be refused", it);
        if !message.count fail("--set % needs a message", it);
    }

    ok, message := format_apply_setting(*applied, "indent_width=17");
    if ok || message != "indent_width must be an integer from 1 to 16" fail("range message: %", message);
}

// The cases jailint's Rust tests run too: `pattern<TAB>path<TAB>match|nomatch`.
check_shared_globs :: () {
    text, ok := read_entire_file("../../../../tests/glob/cases.txt");
    if !ok {
        fail("cannot read the shared glob cases");
        return;
    }

    count := 0;

    for line: split(text, "\n") {
        if !line.count || line[0] == #char "#" continue;

        parts := split(line, "\t");
        if parts.count != 3 {
            fail("bad glob case: %", line);
            continue;
        }

        got := format_glob_match(parts[0], parts[1]);
        if got != (parts[2] == "match") fail("glob(%, %) = %, want %", parts[0], parts[1], got, parts[2]);

        count += 1;
    }

    if count < 30 fail("expected the shared glob cases, found %", count);
}

// The TOML forms a config may use besides `key = value` under `[format]`.
check_toml_forms :: () {
    literal, literal_ok, literal_message := parse_config("[format]\nbrace_style = 'preserve'\nignore = ['a#b/**', \"c\\\"d\"] # 'x'\n");
    if !literal_ok fail("literal strings rejected: %", literal_message);

    if literal.brace_style != .PRESERVE || literal.ignore.count != 2 || literal.ignore[0] != "a#b/**" || literal.ignore[1] != "c\"d" {
        fail("strings parsed wrong: %", literal);
    }

    quoted, quoted_ok, quoted_message := parse_config("[format]\n\"indent_width\" = 3\n'max_width' = 90\n");
    if !quoted_ok || quoted.indent_width != 3 || quoted.max_width != 90 fail("quoted keys: % %", quoted_message, quoted);

    // Every way of writing `[format] indent_width = 2` in a jacket.toml.
    for string.["[format]\nindent_width = 2\n", "[ \"format\" ]\n\"indent_width\" = 2\n", "format.indent_width = 2\n", "format . 'indent_width' = 2\n", "[format]\nindent_width = 2\n\n[lint]\nmax_nesting = 3\n", "format = { indent_width = 2 }\n", "format = {\n  max_width = 80, # wide\n  indent_width = 2,\n}\n", "[project]\nentry = \"a.jai\"\n\n[format.sub]\nx = 1\n\n[other]\nformat.indent_width = 9\n[format]\nindent_width = 2\n"] {
        parsed, ok, message := parse_config(it);
        if !ok fail("jacket.toml rejected: % in %", message, it);
        if ok && parsed.indent_width != 2 fail("jacket.toml form not read: %", it);
    }

    table_form, table_ok, table_message := parse_config("format = { ignore = ['a/**', \"b.jai\"], brace_style = 'preserve', colour = 1, extra.deep = { x = 1 } }\n");
    if !table_ok fail("inline table rejected: %", table_message);

    if table_form.brace_style != .PRESERVE || table_form.ignore.count != 2 || table_form.unknown_keys.count != 2 || table_form.unknown_keys[0] != "colour" || table_form.unknown_keys[1] != "extra.deep" {
        fail("inline table parsed wrong: %", table_form);
    }

    dotted_unknown, dotted_ok := parse_config("format.extra.deep = 1\nother.format = 1\n");
    if !dotted_ok || dotted_unknown.unknown_keys.count != 1 || dotted_unknown.unknown_keys[0] != "extra.deep" fail("dotted unknown key: %", dotted_unknown);

    // Other tables are skipped, whatever they hold.
    others, others_ok := parse_config("[project]\nname = 'a = b'\n[lint]\nexclude = [\"x\"]\nrules = { format = 1 }\n");
    if !others_ok || others.indent_width != 4 || others.unknown_keys.count fail("other tables should be skipped");

    for string.["format = 3\n", "format = { indent_width = x }\n", "format = { indent_width = 2\n", "format.indent_width = 'two'\n", "[format\nindent_width = 2\n", "[format]\nindent_width = 2 3\n", "[format]\nbrace_style = 'weird'\n", "[format]\nignore = [ 'a\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad jacket.toml accepted: %", it);
    }

}

// The formatter is written in the style it enforces.
check_self_formatted :: () {
    for path: SOURCES {
        source, ok := read_entire_file(path);
        if !ok {
            fail("cannot read %", path);
            continue;
        }

        output, formatted, error := format_source(source);
        if !formatted {
            fail("%: %", path, error);
            continue;
        }

        if output != source report_difference(path, source, output);
    }
}
