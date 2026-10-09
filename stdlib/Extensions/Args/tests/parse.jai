// Args parse matrix: every syntax form, every field type, defaults, @env, @range, relations,
// subcommands, and the errors each failure produces (kind, message, suggestion, usage).
//
//     jaic run stdlib/Extensions/Args/tests/parse.jai

#import "Basic";
#import "String";

#load "data/fixtures.jai";

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

same_strings :: (a: [] string, b: [] string) -> bool {
    if a.count != b.count return false;
    for a if it != b[it_index] return false;
    return true;
}

with_program :: (name: string, args: [] string) -> [] string {
    list: [..] string;
    array_add(*list, name);
    for args array_add(*list, it);
    return list;
}

// Runs one parse and checks that it succeeded.
cli_ok :: (args: ..string) -> Cli, Args.Is_Set(Cli) {
    cli, error, set := Args.try_parse(Cli, HELP, with_program("jaifmt", args));
    if error.kind != .NONE fail("%: unexpected error %: %", join(..args, " "), error.kind, error.message);
    return cli, set;
}

// Runs one parse and checks that it failed with this kind and message.
cli_fails :: (kind: Args.Error_Kind, message: string, args: ..string) -> Args.Error {
    _, error := Args.try_parse(Cli, HELP, with_program("jaifmt", args));
    if error.kind != kind fail("%: expected % but got % (%)", join(..args, " "), kind, error.kind, error.message);
    else if message.count && error.message != message fail("%: message was `%`, expected `%`", join(..args, " "), error.message, message);
    return error;
}

test_defaults_and_syntax :: () {
    cli, set := cli_ok();
    expect(!cli.check && cli.jobs == 4 && cli.color == .AUTO && cli.config == "" && cli.verbose == 0 && cli.paths.count == 0, "defaults");
    expect(!set.check && !set.jobs && !set.color && !set.config && !set.verbose && !set.paths, "nothing is set by default");

    cli, set = cli_ok("--check");
    expect(cli.check && set.check, "--check");
    cli, set = cli_ok("-c");
    expect(cli.check && set.check, "-c");
    cli, set = cli_ok("--check", "--no-check");
    expect(!cli.check && set.check, "--no-check after --check wins and counts as set");

    for form: string.["--jobs=8", "-j8", "-j=8"] {
        cli, set = cli_ok(form);
        expect(cli.jobs == 8 && set.jobs, tprint("jobs form %", form));
    }
    cli, set = cli_ok("--jobs", "9");
    expect(cli.jobs == 9, "--jobs 9");
    cli, set = cli_ok("-j", "10");
    expect(cli.jobs == 10, "-j 10");
    cli, set = cli_ok("-j", "3", "-j", "5");
    expect(cli.jobs == 5, "a repeated scalar option keeps the last value");

    cli, set = cli_ok("-cj7");
    expect(cli.check && cli.jobs == 7, "bundled flag then option with attached value");
    cli, set = cli_ok("-cj", "6");
    expect(cli.check && cli.jobs == 6, "bundled flag then option taking the next argument");
    cli, set = cli_ok("-vvv");
    expect(cli.verbose == 3 && set.verbose, "-vvv counts");
    cli, set = cli_ok("-v", "--verbose", "-cv");
    expect(cli.verbose == 3 && cli.check, "counts add up across forms");

    cli, set = cli_ok("a.jai", "--check", "b.jai", "c.jai");
    expect(same_strings(cli.paths, .["a.jai", "b.jai", "c.jai"]) && set.paths, "positionals interleaved with options");
    cli, set = cli_ok("--", "--check", "-x", "-");
    expect(same_strings(cli.paths, .["--check", "-x", "-"]) && !cli.check, "-- ends options; - is a value");
    cli, set = cli_ok("-");
    expect(same_strings(cli.paths, .["-"]), "- alone is a positional");
    cli, set = cli_ok("--config", "-weird");
    expect(cli.config == "-weird", "a value may start with a dash");
    cli, set = cli_ok("--config=");
    expect(cli.config == "" && set.config, "an empty =value is a value");
    cli, set = cli_ok("--config=a=b");
    expect(cli.config == "a=b", "only the first = splits");
    cli, set = cli_ok("-c", "--", "x");
    expect(cli.check && same_strings(cli.paths, .["x"]), "options before --");

    cli, set = cli_ok("--color", "never");
    expect(cli.color == .NEVER && set.color, "enum value");
    cli, set = cli_ok("--color=ALWAYS");
    expect(cli.color == .ALWAYS, "enum values are case-insensitive");
}

test_env :: () {
    cli, error, set := Args.try_parse(Cli, HELP, with_program("jaifmt", .[]), .["JAIFMT_JOBS=12"]);
    expect(error.kind == .NONE && cli.jobs == 12 && set.jobs, "@env fills an option that was not given");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["-j", "3"]), .["JAIFMT_JOBS=12"]);
    expect(error.kind == .NONE && cli.jobs == 3, "the command line beats @env");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .[]), .["JAIFMT_JOBS="]);
    expect(error.kind == .NONE && cli.jobs == 4 && !set.jobs, "an empty variable counts as unset");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .[]), .["JAIFMT_JOBS=lots"]);
    expect(error.kind == .INVALID_VALUE && error.message == "invalid value `lots` for `$JAIFMT_JOBS`: expected an integer", tprint("bad @env value: %", error.message));
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .[]), .["JAIFMT_JOBS=999"]);
    expect(error.kind == .OUT_OF_RANGE, "@env values are range-checked");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .[]), .["UNRELATED=1", "JAIFMT_JOBS_X=2", "JAIFMT_JOBS=5"]);
    expect(cli.jobs == 5, "@env matches whole names");

    tool, tool_error, tool_set := Args.try_parse(Tool, TOOL_HELP, with_program("tool", .["build"]), .["BUILD_TARGET=wasm"]);
    expect(tool_error.kind == .NONE && tool.build.target == "wasm" && tool_set.build.target, "@env inside a subcommand");
}

test_errors :: () {
    error := cli_fails(.UNKNOWN_OPTION, "unknown option `--chek`", "--chek");
    expect(error.suggestion == "--check" && error.option == "--chek", "typo suggestion");
    expect(error.usage == "jaifmt [OPTIONS] [PATHS]..." && error.command == "jaifmt", "usage line and command");
    error = cli_fails(.UNKNOWN_OPTION, "unknown option `--zzzzzz`", "--zzzzzz");
    expect(error.suggestion == "", "no suggestion when nothing is close");
    error = cli_fails(.UNKNOWN_OPTION, "unknown option `--no-check2`", "--no-check2");
    error = cli_fails(.UNKNOWN_OPTION, "unknown option `-x`", "-x");
    error = cli_fails(.UNKNOWN_OPTION, "unknown option `-z`", "-cz");
    error = cli_fails(.UNKNOWN_OPTION, "unknown option `--no-config`", "--no-config");
    error = cli_fails(.MISSING_VALUE, "option `--jobs` needs a value", "--jobs");
    error = cli_fails(.MISSING_VALUE, "option `-j` needs a value", "-j");
    error = cli_fails(.MISSING_VALUE, "option `--color` needs a value", "--check", "--color");
    error = cli_fails(.UNEXPECTED_VALUE, "option `--check` does not take a value", "--check=yes");
    error = cli_fails(.UNEXPECTED_VALUE, "option `--verbose` does not take a value", "--verbose=3");
    error = cli_fails(.OUT_OF_RANGE, "value `0` for `--jobs` is out of range (1..64)", "--jobs", "0");
    error = cli_fails(.OUT_OF_RANGE, "value `65` for `-j` is out of range (1..64)", "-j65");
    error = cli_fails(.OUT_OF_RANGE, "value `-1` for `--jobs` is out of range (1..64)", "--jobs=-1");
    error = cli_fails(.INVALID_VALUE, "invalid value `abc` for `--jobs`: expected an integer", "--jobs", "abc");
    error = cli_fails(.INVALID_VALUE, "invalid value `1.5` for `--jobs`: expected an integer", "--jobs", "1.5");
    error = cli_fails(.INVALID_VALUE, "invalid value `` for `--jobs`: expected an integer", "--jobs=");
    error = cli_fails(.INVALID_VALUE, "value `99999999999999999999` for `--jobs` does not fit in s64", "--jobs", "99999999999999999999");
    error = cli_fails(.INVALID_VALUE, "invalid value `nver` for `--color` (possible values: auto, always, never; did you mean `never`?)", "--color", "nver");
    error = cli_fails(.INVALID_VALUE, "invalid value `purple` for `--color` (possible values: auto, always, never)", "--color=purple");

    // The first problem wins; later arguments are not read.
    error = cli_fails(.UNKNOWN_OPTION, "unknown option `--first`", "--first", "--second");

    cli: Cli;
    set: Args.Is_Set(Cli);
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["--help"]));
    expect(error.kind == .HELP && begins_with(error.text, "Format Jai source files.\n\nUsage: jaifmt [OPTIONS] [PATHS]..."), "--help");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["-h"]));
    expect(error.kind == .HELP, "-h");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["-ch"]));
    expect(error.kind == .HELP && cli.check, "-h inside a bundle");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["--check", "--help", "--bogus"]));
    expect(error.kind == .HELP, "--help wins over later problems");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["--version"]));
    expect(error.kind == .VERSION && error.text == "jaifmt 1.2.3\n", "--version");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["-V"]));
    expect(error.kind == .VERSION, "-V");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("/usr/local/bin/jaifmt.exe", .["--version"]));
    expect(error.text == "jaifmt 1.2.3\n", "the program name is argv[0]'s basename without .exe");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["--help", "--"]), .[]);
    expect(error.kind == .HELP, "--help before --");
    cli, error, set = Args.try_parse(Cli, HELP, with_program("jaifmt", .["--", "--help"]));
    expect(error.kind == .NONE && same_strings(cli.paths, .["--help"]), "--help after -- is a value");
}

test_kinds :: () {
    parse :: (args: ..string) -> Kinds, Args.Error, Args.Is_Set(Kinds) {
        return Args.try_parse(Kinds, KINDS_HELP, with_program("kinds", args));
    }

    kinds, error, set := parse();
    expect(error.kind == .NONE && !kinds.flag && kinds.on_by_default && kinds.signed == -5 && kinds.ratio == 0.5, "Kinds defaults");

    kinds, error, set = parse("--flag", "--no-on-by-default", "--small", "200", "--wide=-9000000000", "--signed", "-300", "--ratio", "2.25", "--single=3.5", "--name", "héllo wörld", "--color=never");
    expect(error.kind == .NONE, tprint("Kinds values: %", error.message));
    expect(kinds.flag && !kinds.on_by_default && kinds.small == 200 && kinds.wide == -9000000000 && kinds.signed == -300, "integers");
    expect(kinds.ratio == 2.25 && kinds.single == 3.5 && kinds.name == "héllo wörld" && kinds.color == .NEVER, "floats, strings, enums");
    expect(set.flag && set.on_by_default && set.small && set.ratio && !set.shades, "Kinds is_set");

    kinds, error, set = parse("-i", "1", "--ids", "2", "--ids=3", "--tags", "a", "--tags=b", "--shades", "never", "--shades=always");
    expect(error.kind == .NONE && same_strings(string.["x"], string.["x"]), "lists parse");
    expect(kinds.ids.count == 3 && kinds.ids[0] == 1 && kinds.ids[1] == 2 && kinds.ids[2] == 3 && set.ids, "repeatable integer option");
    expect(same_strings(kinds.tags, .["a", "b"]), "repeatable string option");
    expect(kinds.shades.count == 2 && kinds.shades[0] == .NEVER && kinds.shades[1] == .ALWAYS, "repeatable enum option");

    kinds, error, set = parse("--level", "-3");
    expect(error.kind == .UNKNOWN_OPTION && error.suggestion == "--lvl", "@long renames the option, and the old name is a typo");
    kinds, error, set = parse("--lvl", "-3", "--tolerance", "1.5");
    expect(error.kind == .NONE && kinds.level == -3 && kinds.tolerance == 1.5, "range edges are inclusive");
    kinds, error, set = parse("--lvl=-4");
    expect(error.kind == .OUT_OF_RANGE && error.message == "value `-4` for `--lvl` is out of range (-3..3)", tprint("%", error.message));
    kinds, error, set = parse("--tolerance=1.6");
    expect(error.kind == .OUT_OF_RANGE && error.message == "value `1.6` for `--tolerance` is out of range (0.0..1.5)", tprint("%", error.message));
    kinds, error, set = parse("--tolerance=fast");
    expect(error.kind == .INVALID_VALUE && error.message == "invalid value `fast` for `--tolerance`: expected a number", tprint("%", error.message));

    kinds, error, set = parse("--output-dir", "out", "-o", "out2");
    expect(error.kind == .NONE && kinds.output_dir == "out2", "a field with an underscore is --output-dir");
    kinds, error, set = parse("--secret", "x");
    expect(error.kind == .NONE && kinds.secret == "x", "a hidden option still parses");

    kinds, error, set = parse("--small", "256");
    expect(error.kind == .INVALID_VALUE && error.message == "value `256` for `--small` does not fit in u8", tprint("%", error.message));
    kinds, error, set = parse("--small=-1");
    expect(error.kind == .INVALID_VALUE && error.message == "value `-1` for `--small` does not fit in u8", tprint("%", error.message));
    kinds, error, set = parse("--wide", "-5");
    expect(error.kind == .NONE && kinds.wide == -5, "a negative number is a value");
    kinds, error, set = parse("--no-name");
    expect(error.kind == .UNKNOWN_OPTION, "only bool flags have a --no- form");
    kinds, error, set = parse("--no-flag", "--flag");
    expect(error.kind == .NONE && kinds.flag, "--no-flag then --flag");
}

test_relations :: () {
    parse :: (args: ..string) -> Relations, Args.Error, Args.Is_Set(Relations) {
        return Args.try_parse(Relations, RELATIONS_HELP, with_program("rel", args));
    }
    r, error, set := parse("--mode", "m", "-i", "f");
    expect(error.kind == .NONE && r.input == "f" && r.mode == "m", "ok");
    r, error, set = parse("-i", "f");
    expect(error.kind == .MISSING_REQUIRED && error.message == "the option `--mode` is required" && error.option == "--mode", tprint("%", error.message));
    r, error, set = parse("--mode=m", "-i", "f", "--stdin");
    expect(error.kind == .CONFLICT && error.message == "`--input` cannot be used with `--stdin`", tprint("%", error.message));
    r, error, set = parse("--mode=m", "--stdin");
    expect(error.kind == .REQUIRES && error.message == "`--stdin` requires `--format`", tprint("%", error.message));
    r, error, set = parse("--mode=m", "--stdin", "--format", "x");
    expect(error.kind == .NONE && r.stdin && r.format == "x", "requires satisfied");
    r, error, set = parse("--mode=m", "a", "b");
    expect(error.kind == .NONE && same_strings(r.paths, .["a", "b"]), "positionals");
}

test_positionals :: () {
    parse :: (args: ..string) -> Copy, Args.Error, Args.Is_Set(Copy) {
        return Args.try_parse(Copy, COPY_HELP, with_program("cp", args));
    }
    c, error, set := parse("a", "b");
    expect(error.kind == .NONE && c.source == "a" && c.target == "b" && c.extra.count == 0 && set.source && set.target && !set.extra, "two positionals");
    c, error, set = parse("a", "b", "c", "d", "-f");
    expect(error.kind == .NONE && c.force && same_strings(c.extra, .["c", "d"]), "the last positional collects the rest");
    c, error, set = parse("-f", "a");
    expect(error.kind == .MISSING_REQUIRED && error.message == "the argument `<DEST>` is required", tprint("%", error.message));
    c, error, set = parse();
    expect(error.kind == .MISSING_REQUIRED && error.message == "the argument `<SOURCE>` is required", tprint("%", error.message));
    expect(error.usage == "cp [OPTIONS] <SOURCE> <DEST> [EXTRA]...", tprint("%", error.usage));
    c, error, set = parse("a", "b", "--", "-f");
    expect(error.kind == .NONE && same_strings(c.extra, .["-f"]) && !c.force, "-- then a dash value");

    // More positionals than declared, with no list to take them.
    Few :: struct {
        one: string; @positional
    }
    _, few_error := Args.try_parse(Few, Args.Help(Few).{ one = "x" }, string.["few", "a", "b"]);
    expect(few_error.kind == .UNEXPECTED_ARGUMENT && few_error.message == "unexpected argument `b`", tprint("%", few_error.message));
}

test_subcommands :: () {
    parse :: (args: ..string) -> Tool, Args.Error, Args.Is_Set(Tool) {
        return Args.try_parse(Tool, TOOL_HELP, with_program("tool", args));
    }
    t, error, set := parse("build", "-r", "--target", "x", "a", "b");
    expect(error.kind == .NONE && t.command == .BUILD && t.build.release && t.build.target == "x", "build");
    expect(same_strings(t.build.files, .["a", "b"]) && set.command && set.build.release && set.build.target && set.build.files, "build is_set");
    expect(!set.run.program, "the other subcommand stays unset");

    t, error, set = parse("run", "prog", "--", "--flag", "x");
    expect(error.kind == .NONE && t.command == .RUN && t.run.program == "prog" && same_strings(t.run.arguments, .["--flag", "x"]), "run with -- arguments");
    t, error, set = parse("remote-add", "origin", "url");
    expect(error.kind == .NONE && t.command == .REMOTE_ADD && t.remote_add.name == "origin" && t.remote_add.url == "url", "a field named remote_add is the subcommand remote-add");

    t, error, set = parse("-vv", "build", "-v", "--config=c.toml", "x");
    expect(error.kind == .NONE && t.verbose == 3 && t.config == "c.toml" && set.verbose && set.config, "@global options before and after the subcommand");
    t, error, set = parse("build", "-q");
    expect(error.kind == .UNKNOWN_OPTION && error.message == "unknown option `-q`", "an option that is not @global is not accepted after a subcommand");

    t, error, set = parse();
    expect(error.kind == .MISSING_COMMAND && error.message == "a subcommand is required" && error.usage == "tool [OPTIONS] <COMMAND>", tprint("%", error.message));
    t, error, set = parse("-v");
    expect(error.kind == .MISSING_COMMAND, "options alone are not a command");
    t, error, set = parse("bild");
    expect(error.kind == .UNKNOWN_COMMAND && error.message == "unrecognized subcommand `bild`" && error.suggestion == "build", tprint("%", error.message));
    t, error, set = parse("-q", "-v", "build");
    expect(error.kind == .CONFLICT, "conflicts are checked on the command that declares them");
    t, error, set = parse("run");
    expect(error.kind == .MISSING_REQUIRED && error.message == "the argument `<PROGRAM>` is required", tprint("%", error.message));
    expect(error.command == "tool run" && error.usage == "tool run [OPTIONS] <PROGRAM> [ARGUMENTS]...", tprint("% / %", error.command, error.usage));
    t, error, set = parse("build", "--rleease");
    expect(error.kind == .UNKNOWN_OPTION && error.suggestion == "--release" && error.command == "tool build" && error.usage == "tool build [OPTIONS] [FILES]...", "subcommand typo");
    t, error, set = parse("build", "--help");
    expect(error.kind == .HELP && begins_with(error.text, "Compile the project\n\nUsage: tool build [OPTIONS] [FILES]..."), "subcommand help");
    t, error, set = parse("build", "--version");
    expect(error.kind == .UNKNOWN_OPTION, "--version is the program's, not a subcommand's");
    t, error, set = parse("--version");
    expect(error.kind == .VERSION && error.text == "tool 0.1.0\n", "root version");

    o, o_error, o_set := Args.try_parse(Optional_Tool, OPTIONAL_HELP, string.["opt"]);
    expect(o_error.kind == .NONE && !o_set.command, "an @optional selector may be left out");
    o, o_error, o_set = Args.try_parse(Optional_Tool, OPTIONAL_HELP, string.["opt", "-n", "init", "here"]);
    expect(o_error.kind == .NONE && o.dry_run && o.command == .INIT && o.init.path == "here" && o_set.command && o_set.init.path, "optional selector with a subcommand");
    o, o_error, o_set = Args.try_parse(Optional_Tool, OPTIONAL_HELP, string.["opt", "clean", "--all"]);
    expect(o_error.kind == .NONE && o.command == .CLEAN && o.clean.all, "second subcommand");
}

test_shell_option :: () {
    Completer :: struct {
        completions: Args.Shell;
    }
    c, error, set := Args.try_parse(Completer, Args.Help(Completer).{ completions = "Print a completion script" }, string.["x", "--completions", "powershell"]);
    expect(error.kind == .NONE && c.completions == .POWERSHELL && set.completions, "Args.Shell works as an option type");
}

main :: () {
    test_shell_option();
    test_defaults_and_syntax();
    test_env();
    test_errors();
    test_kinds();
    test_relations();
    test_positionals();
    test_subcommands();
    if failures {
        print("% failures\n", failures);
        exit(1);
    }
    print("ok\n");
}
