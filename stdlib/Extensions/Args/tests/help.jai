// Args help output against golden texts (data/golden.jai), plus the colour rules, the
// subcommand help pages and the error report layout.
//
//     jaic run stdlib/Extensions/Args/tests/help.jai
//
// After an intended layout change, regenerate the goldens with data/bless.jai.

#import "Basic";
#import "String";

#load "data/outputs.jai";
#load "data/golden.jai";

failures := 0;

fail :: (format: string, args: ..Any) {
    print("FAIL: ");
    print(format, ..args);
    print("\n");
    failures += 1;
}

expect :: (condition: bool, what: string) {
    if !condition fail("%", what);
}

same_text :: (name: string, actual: string, expected: string) {
    if actual == expected return;
    fail("% differs from its golden text; first difference at byte %\n--- actual ---\n%\n--- expected ---\n%", name, first_difference(actual, expected), actual, expected);
}

first_difference :: (a: string, b: string) -> s64 {
    for 0..min(a.count, b.count) - 1 if a[it] != b[it] return it;
    return min(a.count, b.count);
}

golden :: (name: string) -> string {
    if name == {
        case "CLI_HELP"; return CLI_HELP;
        case "CLI_HELP_COLOR"; return CLI_HELP_COLOR;
        case "KINDS_HELP_TEXT"; return KINDS_HELP_TEXT;
        case "COPY_HELP_TEXT"; return COPY_HELP_TEXT;
        case "TOOL_HELP_TEXT"; return TOOL_HELP_TEXT;
        case "OPTIONAL_HELP_TEXT"; return OPTIONAL_HELP_TEXT;
        case "TOOL_BUILD_HELP"; return TOOL_BUILD_HELP;
        case "TOOL_REMOTE_HELP"; return TOOL_REMOTE_HELP;
    }
    return "";
}

main :: () {
    for help_outputs() same_text(it.name, it.text, golden(it.name));

    // The text --help returns is the same page that help_text renders, without colour in a
    // test run that has colour off.
    Args.set_color_mode(.NEVER);
    argv := string.["jaifmt", "--help"];
    cli, error, set := Args.try_parse(Cli, HELP, argv);
    same_text("--help", error.text, CLI_HELP);

    // Subcommand pages.
    same_text("tool -v build -h", tool_help_page("-v", "build", "-h"), TOOL_BUILD_HELP);

    // Colour follows the mode: always puts the codes in, never leaves them out.
    Args.set_color_mode(.ALWAYS);
    cli, error, set = Args.try_parse(Cli, HELP, argv);
    expect(contains(error.text, "\e[1;4mUsage:\e[0m"), "colour always: headings are styled");
    Args.set_color_mode(.NEVER);
    cli, error, set = Args.try_parse(Cli, HELP, argv);
    expect(!contains(error.text, "\e"), "colour never: no escape codes");

    Args.set_color_mode(.AUTO);
    expect(Args.color_enabled(.OUTPUT) == Args.color_enabled(.OUTPUT), "colour auto is stable");

    check_error_layout();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }
    print("ok\n");
}

// `Args.report` prints to stderr; the layout itself is what the error carries.
check_error_layout :: () {
    Args.set_color_mode(.NEVER);
    _, error := Args.try_parse(Cli, HELP, string.["jaifmt", "--chek"]);
    Args.report(error); // to stderr; the fields below are what it prints
    expect(error.kind == .UNKNOWN_OPTION, "error layout setup");
    expect(error.message == "unknown option `--chek`" && error.suggestion == "--check" && error.usage == "jaifmt [OPTIONS] [PATHS]..." && error.command == "jaifmt", "error fields feed the report");
}
