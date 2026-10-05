// jaifmt golden tests: every tools/jaifmt/tests/<name>.in.jai formats to <name>.out.jai (with
// <name>.toml as its config when present), formatting is idempotent and keeps the token stream,
// malformed input is refused, and the formatter's own sources are already formatted.
//
//     jaic run tests/stdlib/jaifmt-golden.jai              check
//     jaic run tests/stdlib/jaifmt-golden.jai -- --bless   rewrite the .out.jai files

#import "Basic";
#import "String";
#import "File";
#import "File_Utilities";
#import "Sort";

#load "../../tools/jaifmt/lexer.jai";
#load "../../tools/jaifmt/config.jai";
#load "../../tools/jaifmt/format.jai";

CASES :: "../../tools/jaifmt/tests";
SOURCES :: "../../tools/jaifmt";

failures := 0;

fail :: (format: string, args: .. Any) {
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
        config: Config;
        toml, has_config := read_entire_file(tprint("%.toml", base), log_errors = false);
        if has_config {
            ok, message := parse_config(toml, *config);
            if !ok { fail("%.toml: %", base, message); continue; }
        }
        input, read_ok := read_entire_file(tprint("%.in.jai", base));
        if !read_ok { fail("cannot read %.in.jai", base); continue; }

        result := format_source(input, config);
        if !result.ok { fail("%.in.jai: %", base, result.error); continue; }

        again := format_source(result.output, config);
        if !again.ok || again.output != result.output fail("%: formatting is not idempotent", base);

        expected_path := tprint("%.out.jai", base);
        if bless {
            write_entire_file(expected_path, result.output.data, result.output.count);
            continue;
        }
        expected, expected_ok := read_entire_file(expected_path, log_errors = false);
        if !expected_ok { fail("missing %; run with -- --bless", expected_path); continue; }
        if expected != result.output report_difference(base, expected, result.output);
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
    config: Config;
    bad :: string.[
        "f :: () { x := 1;\n",          // unclosed block
        "f :: () { x := (1; }\n",       // mismatched bracket
        "s := \"unterminated;\n",
        "t := #string END\nno terminator\n",
        "/* unterminated comment\n",
        "x := 1 \x01 2;\n",              // a byte the lexer does not know
    ];
    for bad {
        result := format_source(it, config);
        if result.ok fail("malformed input was formatted: %", it);
    }
}

// The guard itself: different token streams, line breaks or comments are reported.
check_equivalence_guard :: () {
    same, why := check_equivalent("a := b+c; // x\n", "a := b + c;  // x\n");
    if !same fail("equivalent sources rejected: %", why);
    pairs :: string.[
        "x := a & &b;\n",        "x := a &&b;\n",
        "a := 1;\n",             "a := 2;\n",
        "x := a // note\n;\n",   "x := a;\n",
        "x := a;\n-b;\n",        "x := a; -b;\n",
        "s := \"a b\";\n",       "s := \"a  b\";\n",
        // Directive flags are only flags when written against the directive.
        "k :: #library,system \"k\";\n",  "k :: #library, system \"k\";\n",
        "k :: #library ,system \"k\";\n", "k :: #library,system \"k\";\n",
        "l := #location(x);\n",          "l := #location (x);\n",
    ];
    i := 0;
    while i < pairs.count {
        equivalent := check_equivalent(pairs[i], pairs[i + 1]);
        if equivalent fail("check_equivalent accepted % -> %", pairs[i], pairs[i + 1]);
        i += 2;
    }
}

check_config_parsing :: () {
    config: Config;
    ok, message := parse_config("indent_width = 2 # two\nmax_width = 120\nbrace_style = \"preserve\"\nignore = [\"a/**\", \"b.jai\"]\n", *config);
    if !ok fail("config rejected: %", message);
    if config.indent_width != 2 || config.max_width != 120 || config.brace_style != .PRESERVE || config.ignore.count != 2 {
        fail("config parsed wrong: %", config);
    }
    for string.["indent_width = x\n", "unknown = 1\n", "brace_style = \"weird\"\n", "ignore = \"a\"\n"] {
        bad: Config;
        accepted := parse_config(it, *bad);
        if accepted fail("bad config accepted: %", it);
    }

    config.root = "/repo";
    if !is_ignored(*config, "/repo/a/x/y.jai") fail("`a/**` should ignore /repo/a/x/y.jai");
    if !is_ignored(*config, "/repo/b.jai") fail("`b.jai` should ignore /repo/b.jai");
    if is_ignored(*config, "/repo/c/b.jai") fail("`b.jai` should not ignore /repo/c/b.jai");
    if !glob_match("**/gen_*.jai", "x/y/gen_a.jai") || !glob_match("**/gen_*.jai", "gen_b.jai") fail("`**/` glob");
    if glob_match("*.jai", "dir/a.jai") fail("`*` must not cross directories");
}

// The formatter is written in the style it enforces.
check_self_formatted :: () {
    config: Config;
    for string.["lexer.jai", "config.jai", "format.jai", "main.jai"] {
        path := tprint("%/%", SOURCES, it);
        source, ok := read_entire_file(path);
        if !ok { fail("cannot read %", path); continue; }
        result := format_source(source, config);
        if !result.ok { fail("%: %", path, result.error); continue; }
        if result.output != source report_difference(path, source, result.output);
    }
    this_file, ok := read_entire_file("jaifmt-golden.jai");
    if ok {
        result := format_source(this_file, config);
        if !result.ok || result.output != this_file fail("tests/stdlib/jaifmt-golden.jai is not formatted");
    }
}
