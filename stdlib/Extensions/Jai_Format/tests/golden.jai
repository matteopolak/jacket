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
SOURCES :: string.["../module.jai", "../lexer.jai", "../config.jai", "../format.jai", "../range.jai", "golden.jai", "../../../../jaifmt/main.jai"];

failures := 0;

fail :: (format: string, args: ..Any) {
    print("FAIL: ");
    print(format, ..args);
    print("\n");
    failures += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
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

    check_ranges(bless);
    check_range_edges();
    check_refusals();
    check_unparsed_tokens();
    check_equivalence_guard();
    check_config_parsing();
    check_toml_forms();
    check_self_formatted();
    check_rules_page();

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

// docs/tools/jaifmt-rules.md: every ```` ```jai after ```` block is what the ```` ```jai before ````
// block above it formats to, with the settings written after `after` (`indent_width=2`).
RULES_PAGE :: "../../../../docs/tools/jaifmt-rules.md";

check_rules_page :: () {
    text, ok := read_entire_file(RULES_PAGE);
    if !ok {
        fail("cannot read %", RULES_PAGE);
        return;
    }

    // Odd pieces are the insides of fences: the info line, then the code. Only a fence at the
    // start of a line counts, since an example may show a fence inside a comment.
    pieces := split(text, "\n```");
    pairs := 0;
    i := 1;

    while i + 2 < pieces.count {
        before := tprint("%\n", pieces[i]);
        after := tprint("%\n", pieces[i + 2]);

        if !begins_with(before, "jai before\n") || !begins_with(after, "jai after") {
            i += 2;
            continue;
        }

        info, code := split_line(after);
        _, input := split_line(before);
        config: Format_Config;

        for word: split(info, " ") {
            if word == "jai" || word == "after" || word == "" continue;
            set, message := format_apply_setting(*config, word);
            if !set fail("%: %: %", RULES_PAGE, word, message);
        }

        output, formatted, error := format_source(input, config);
        if !formatted fail("%: %", RULES_PAGE, error);
        else if output != code report_difference(tprint("% (%)", RULES_PAGE, info), code, output);

        pairs += 1;
        i += 4;
    }

    if pairs < 16 fail("% has % before/after pairs; expected every rule and setting", RULES_PAGE, pairs);
}

split_line :: (s: string) -> first: string, rest: string {
    found, first, rest := split_from_left(s, #char "\n");
    if !found return s, "";
    return first, rest;
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

// Code the compiler rejects but the lexer reads, as an editor sends it mid-edit: the formatter
// spaces the tokens and keeps each one whole. These stay inline so no fixture file fails to parse.
check_unparsed_tokens :: () {
    cases :: string.[
        // A second decimal point, or digits after a member `.`, starts a new token.
        "d := 1.2.3;\nx := v.5;\n", "d := 1.2 . 3;\nx := v. 5;\n",
        // A note runs to the next whitespace, so `@@b` is one token.
        "a@@b := 0;\n", "a @@b := 0;\n",
    ];
    i := 0;

    while i < cases.count {
        output, ok, error := format_source(cases[i]);
        if !ok fail("% was refused: %", cases[i], error);
        else if output != cases[i + 1] report_difference(cases[i], cases[i + 1], output);

        i += 2;
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
    config, ok, message := parse_config("[format]\nindent-width = 2 # two\nmax-width = 120\nbrace-style = \"preserve\"\nignore = [\"a/**\", \"b.jai\"]\n");
    if !ok fail("config rejected: %", message);

    if config.indent_width != 2 || config.max_width != 120 || config.brace_style != .PRESERVE || config.ignore.count != 2 {
        fail("config parsed wrong: %", config);
    }

    if !config.blank_lines_between_items fail("blank_lines_between_items should default to true");

    off, off_ok := parse_config("[format]\nblank-lines-between-items = false\n");
    if !off_ok || off.blank_lines_between_items fail("blank-lines-between-items = false not parsed");

    if config.blank_line_after_declarations || config.blank_lines_around_blocks || config.blank_line_between_cases || config.blank_line_before_comments || config.blank_line_after_guards || config.blank_line_before_return {
        fail("the code block blank line options should default to false");
    }

    on, on_ok := parse_config("[format]\nblank-line-after-declarations = true\nblank-lines-around-blocks = true\nblank-line-between-cases = true\nblank-line-before-comments = true\nblank-line-after-guards = true\nblank-line-before-return = true\n");
    if !on_ok || !on.blank_line_after_declarations || !on.blank_lines_around_blocks || !on.blank_line_between_cases || !on.blank_line_before_comments || !on.blank_line_after_guards || !on.blank_line_before_return {
        fail("the code block blank line options were not parsed");
    }

    if !config.format_doc_examples fail("format_doc_examples should default to true");

    docs_off, docs_off_ok := parse_config("[format]\nformat-doc-examples = false\n");
    if !docs_off_ok || docs_off.format_doc_examples fail("format-doc-examples = false not parsed");

    for string.["[format]\nindent-width = x\n", "[format]\nbrace-style = \"weird\"\n", "[format]\nignore = \"a\"\n", "[format]\nblank-lines-between-items = 1\n", "[format]\nblank-lines-around-blocks = yes\n", "[format]\nformat-doc-examples = 1\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad config accepted: %", it);
    }

    jacket, jacket_ok, jacket_message := parse_config("[project]\nentry = [\n  \"a.jai\", # the program\n]\n\n[format]\nindent-width = 2 # two\nignore = [\n  \"a/**\", # first\n  \"b.jai\",\n]\nbrace-style = \"preserve\"\n\n[lint.rules]\nindent-width = \"deny\"\n");
    if !jacket_ok fail("jacket.toml rejected: %", jacket_message);

    if jacket.indent_width != 2 || jacket.brace_style != .PRESERVE || jacket.ignore.count != 2 || jacket.ignore[1] != "b.jai" {
        fail("jacket.toml [format] parsed wrong: %", jacket);
    }

    none, none_ok := parse_config("[lint]\nmax-nesting = 3\n");
    if !none_ok || none.indent_width != 4 fail("a jacket.toml without [format] should give the defaults");

    for string.["[format]\nindent-width = x\n", "[format]\nignore = [\"a\"\n", "[format]\nindent-width\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad jacket.toml accepted: %", it);
    }

    for string.["[format]\nindent-widht = 2\n", "[format]\ncolour = 1\n", "[format.extra]\nx = 1\n", "[doc.format]\ncolour = 1\n"] {
        _, accepted := parse_config(it);
        if accepted fail("unknown key accepted: %", it);
    }

    _, _, near := parse_config("[format]\nindent-widht = 2\n");
    if near != "line 2: unknown setting `format.indent-widht`; did you mean `format.indent-width`?" fail("unknown key error: %", near);

    _, _, far := parse_config("[format]\ncolour = 1\n");
    if far != "line 2: unknown setting `format.colour`" fail("unknown key error: %", far);

    if config.doc_format.count fail("a config without [doc.format] should have no doc overrides");

    docs_format, docs_format_ok, docs_format_message := parse_config("[format]\nindent-width = 2\n[doc]\nurl = \"x\"\n[doc.format]\nindent-width = 3\nblank-line-after-declarations = false\nbrace-style = 'preserve'\n");
    if !docs_format_ok fail("[doc.format] rejected: %", docs_format_message);

    if docs_format.indent_width != 2 || !docs_format.format_doc_examples || docs_format.ignore.count fail("[doc.format] must not change the file's own settings");
    if docs_format.doc_format.count != 3 fail("[doc.format] overrides: %", docs_format.doc_format);

    _, _, doc_near := parse_config("[doc.format]\nindent-widht = 2\n");
    if doc_near != "line 2: unknown setting `doc.format.indent-widht`; did you mean `doc.format.indent-width`?" fail("doc unknown key error: %", doc_near);

    docs_inline, docs_inline_ok := parse_config("doc.format.max-width = 60\ndoc.format.blank-lines-around-blocks = true\n");
    if !docs_inline_ok || docs_inline.doc_format.count != 2 fail("inline and dotted [doc.format] forms not parsed");

    for string.["[doc.format]\nindent-width = x\n", "[doc.format]\nbrace-style = \"weird\"\n", "[doc.format]\nblank-line-before-return = 1\n", "[doc]\nformat = 3\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad [doc.format] accepted: %", it);
    }

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

    for string.["indent-width=2", "brace-style=\"preserve\"", "blank-line-before-return = true", "max-width=80"] {
        ok, message := format_apply_setting(*applied, it);
        if !ok fail("--set %: %", it, message);
    }

    doc_ok, doc_message := format_apply_setting(*applied, "doc.format.blank-line-before-return=false");
    if !doc_ok || applied.doc_format.count != 1 fail("--set doc.format.key: %", doc_message);

    doc_ok, doc_message = format_apply_setting(*applied, "doc.format.ignore=[]");
    if doc_ok fail("--set doc.format.ignore should be refused");

    if applied.indent_width != 2 || applied.brace_style != .PRESERVE || !applied.blank_line_before_return || applied.max_width != 80 fail("applied settings wrong: %", applied);

    for string.["indent-width=0", "indent-width=x", "colour=1", "indent-width", "brace-style=same_line", "blank-lines-between-items=1"] {
        ok, message := format_apply_setting(*applied, it);
        if ok fail("--set % should be refused", it);
        if !message.count fail("--set % needs a message", it);
    }

    ok, message := format_apply_setting(*applied, "indent-width=17");
    if ok || message != "indent-width must be an integer from 1 to 16" fail("range message: %", message);
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
    literal, literal_ok, literal_message := parse_config("[format]\nbrace-style = 'preserve'\nignore = ['a#b/**', \"c\\\"d\"] # 'x'\n");
    if !literal_ok fail("literal strings rejected: %", literal_message);

    if literal.brace_style != .PRESERVE || literal.ignore.count != 2 || literal.ignore[0] != "a#b/**" || literal.ignore[1] != "c\"d" {
        fail("strings parsed wrong: %", literal);
    }

    quoted, quoted_ok, quoted_message := parse_config("[format]\n\"indent-width\" = 3\n'max-width' = 90\n");
    if !quoted_ok || quoted.indent_width != 3 || quoted.max_width != 90 fail("quoted keys: % %", quoted_message, quoted);

    // Every way of writing `[format] indent_width = 2` in a jacket.toml.
    for string.["[format]\nindent-width = 2\n", "[ \"format\" ]\n\"indent-width\" = 2\n", "format.indent-width = 2\n", "format . 'indent-width' = 2\n", "[format]\nindent-width = 2\n\n[lint]\nmax-nesting = 3\n", "format = { indent-width = 2 }\n", "format = {\n  max-width = 80, # wide\n  indent-width = 2,\n}\n", "[project]\nentry = \"a.jai\"\n\n[sub]\nx = 1\n\n[other]\nformat.indent-width = 9\n[format]\nindent-width = 2\n"] {
        parsed, ok, message := parse_config(it);
        if !ok fail("jacket.toml rejected: % in %", message, it);
        if ok && parsed.indent_width != 2 fail("jacket.toml form not read: %", it);
    }

    table_form, table_ok, table_message := parse_config("format = { ignore = ['a/**', \"b.jai\"], brace-style = 'preserve' }\n");
    if !table_ok fail("inline table rejected: %", table_message);

    if table_form.brace_style != .PRESERVE || table_form.ignore.count != 2 {
        fail("inline table parsed wrong: %", table_form);
    }

    _, _, inline_unknown := parse_config("format = { brace-style = 'preserve', colour = 1, extra.deep = { x = 1 } }\n");
    if inline_unknown != "line 1: unknown setting `format.colour`" fail("inline unknown key: %", inline_unknown);

    _, _, dotted_unknown := parse_config("format.extra.deep = 1\nother.format = 1\n");
    if dotted_unknown != "line 1: unknown setting `format.extra.deep`" fail("dotted unknown key: %", dotted_unknown);

    // Other tables are skipped, whatever they hold.
    others, others_ok := parse_config("[project]\nname = 'a = b'\n[lint]\nexclude = [\"x\"]\nrules = { format = 1 }\n");
    if !others_ok || others.indent_width != 4 fail("other tables should be skipped");

    for string.["format = 3\n", "format = { indent-width = x }\n", "format = { indent-width = 2\n", "format.indent-width = 'two'\n", "[format\nindent-width = 2\n", "[format]\nindent-width = 2 3\n", "[format]\nbrace-style = 'weird'\n", "[format]\nignore = [ 'a\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad jacket.toml accepted: %", it);
    }

}

// Each range-fixtures/<name>.in.jai is formatted over the bytes in <name>.range (START:END) and
// compared with <name>.out.jai. Text outside the touched statements must come back unchanged.
check_ranges :: (bless: bool) {
    cases := 0;

    for file_list("range-fixtures") {
        if !ends_with(it, ".in.jai") continue;

        base := slice(it, 0, it.count - ".in.jai".count);
        input, _ := read_entire_file(tprint("%.in.jai", base));
        range_text, _ := read_entire_file(tprint("%.range", base));
        spec := trim(range_text);
        parts := split(spec, ":");
        from := string_to_int(parts[0]);
        to := string_to_int(parts[1]);

        output, ok, error := format_range(input, from, to);

        if !ok {
            fail("range %: %", base, error);
            continue;
        }

        cases += 1;

        again, again_ok := format_range(output, from, to);
        if !again_ok || again != output fail("range %: formatting the range again changed it", base);

        expected_path := tprint("%.out.jai", base);

        if bless {
            write_entire_file(expected_path, output.data, output.count);
            continue;
        }

        expected, found := read_entire_file(expected_path, log_errors = false);
        if !found fail("range %: missing %.out.jai (run with --bless)", base, base);
        else if output != expected fail("range %: output differs\n--- expected\n%\n--- got\n%", base, expected, output);
    }

    if cases < 4 fail("expected the range cases in range-fixtures, found %", cases);

    _, bad_ok := format_range("a :: 1;\n", 5, 2);
    if bad_ok fail("a reversed range was accepted");

    _, past_ok := format_range("a :: 1;\n", 0, 99);
    if past_ok fail("a range past the end was accepted");
}

// Range formatting must preserve surrounding bytes, including line endings and off regions.
check_range_edges :: () {
    expect_range("a:=1;\r\nb  :=  2;\r\n", 0, 1, "a := 1;\r\nb  :=  2;\r\n");
    expect_range("a::1; b::2;\nc  ::  3;\n", 7, 8, "a :: 1;\nb :: 2;\nc  ::  3;\n");
    expect_range("#scope_file\nx:=1;\n", 0, 1, "#scope_file\nx:=1;\n");

    off :: "// jaifmt: off\nx:=1;\n// jaifmt: on\n";
    source := tprint("%y:=2;\n", off);

    expect_range(source, 15, 16, source);
    expect_range(source, off.count, off.count + 1, tprint("%y := 2;\n", off));

    for string.["s := \"unterminated;\n", "main :: () {\nx:=1;\n"] {
        output, ok, error := format_range(it, 0, 1);
        defer free(output);
        if ok || !error.count fail("malformed range was accepted: %", it);
    }
}

expect_range :: (source: string, from: s64, to: s64, expected: string) {
    output, ok, error := format_range(source, from, to);
    defer free(output);
    if !ok fail("range %:% failed: %", from, to, error);
    else if output != expected fail("range %:% changed the wrong bytes\nexpected: %\nactual: %", from, to, expected, output);
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
