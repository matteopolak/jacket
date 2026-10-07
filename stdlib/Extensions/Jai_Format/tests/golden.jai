// Jai_Format golden tests: every cases/<name>.in.jai formats to <name>.out.jai (with
// <name>.toml as its config when present), formatting is idempotent and keeps the token stream,
// malformed input is refused, and the formatter's own sources are already formatted.
//
//     jaic run stdlib/Extensions/Jai_Format/tests/golden.jai              check
//     jaic run stdlib/Extensions/Jai_Format/tests/golden.jai -- --bless   rewrite the .out.jai files

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Sort";

#import "Extensions/Jai_Format";

CASES :: "cases";
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
    config, ok, message := parse_config("indent_width = 2 # two\nmax_width = 120\nbrace_style = \"preserve\"\nignore = [\"a/**\", \"b.jai\"]\n");
    if !ok fail("config rejected: %", message);
    if config.indent_width != 2 || config.max_width != 120 || config.brace_style != .PRESERVE || config.ignore.count != 2 {
        fail("config parsed wrong: %", config);
    }
    if !config.blank_lines_between_items fail("blank_lines_between_items should default to true");
    off, off_ok := parse_config("blank_lines_between_items = false\n");
    if !off_ok || off.blank_lines_between_items fail("blank_lines_between_items = false not parsed");
    for string.["indent_width = x\n", "unknown = 1\n", "brace_style = \"weird\"\n", "ignore = \"a\"\n", "blank_lines_between_items = 1\n"] {
        _, accepted := parse_config(it);
        if accepted fail("bad config accepted: %", it);
    }

    config.root = "/repo";
    if !format_is_ignored(config, "/repo/a/x/y.jai") fail("`a/**` should ignore /repo/a/x/y.jai");
    if !format_is_ignored(config, "/repo/b.jai") fail("`b.jai` should ignore /repo/b.jai");
    if format_is_ignored(config, "/repo/c/b.jai") fail("`b.jai` should not ignore /repo/c/b.jai");
    if !format_glob_match("**/gen_*.jai", "x/y/gen_a.jai") || !format_glob_match("**/gen_*.jai", "gen_b.jai") fail("`**/` glob");
    if format_glob_match("*.jai", "dir/a.jai") fail("`*` must not cross directories");
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
