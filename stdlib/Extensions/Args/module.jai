// Args: a typed, compile-time-generated command-line parser for jaic programs, in the spirit of
// Rust's clap derive. A jaic extension, not official Jai: other Jai compilers have no such
// module, and it may change between jaic releases. See docs/stdlib/args.md.
//
//     Args :: #import "Extensions/Args";
//
//     Cli :: struct {
//         paths: [..] string; @positional
//         check: bool;       @short=c
//         jobs:  int = 4;    @short=j @env=TOOL_JOBS @range(1,64)
//     }
//     HELP :: Args.Help(Cli).{ about = "Does things.", paths = "Files to read", check = "Only check", jobs = "Parallel jobs" };
//
//     main :: () {
//         cli, set := Args.parse(Cli, HELP);  // --help, --version, errors (exit 2) are handled
//         if set.jobs { ... }                 // set.<field> is true when the user supplied it
//     }
//
// The fields of `Cli` are the options; notes (`@short=c`) refine them. At compile time the
// struct is read once and the parser is written out as plain Jai (see generate.jai), so the
// per-argument work is string comparisons and direct stores into the fields.

// Why parsing stopped. HELP and VERSION are not failures: `error.text` is what to print.
Error_Kind :: enum u8 {
    NONE;
    HELP;
    VERSION;
    UNKNOWN_OPTION;
    UNKNOWN_COMMAND;
    MISSING_VALUE;
    INVALID_VALUE;
    OUT_OF_RANGE;
    UNEXPECTED_VALUE;
    UNEXPECTED_ARGUMENT;
    MISSING_REQUIRED;
    MISSING_COMMAND;
    CONFLICT;
    REQUIRES;
}

Error :: struct {
    kind: Error_Kind;
    message: string; // `unknown option `--chek``
    option: string; // the option or argument the message is about, as written (`--chek`)
    value: string; // the offending value, when there is one
    suggestion: string; // the nearest known name (`--check`), when one is close
    command: string; // the command that failed, `tool build`
    usage: string; // `tool build [OPTIONS] <PATH>`
    text: string; // for .HELP and .VERSION: what to print
}

// Which terminals get colour in help and errors: `AUTO` follows NO_COLOR, FORCE_COLOR,
// CLICOLOR_FORCE, TERM and whether the stream is a terminal.
Color_Mode :: enum u8 {
    AUTO;
    ALWAYS;
    NEVER;
}

color_mode := Color_Mode.AUTO;

set_color_mode :: (mode: Color_Mode) {
    color_mode = mode;
}

Shell :: enum u8 {
    BASH;
    ZSH;
    FISH;
    POWERSHELL;
}

// The help strings for `T`: `about` and `version`, then one string per option, named after the
// field.
// A subcommand's field holds that subcommand's own Help. A field not named in the literal has
// no help; that is a compile-time warning unless the field is `@hidden`.
Help :: struct(T: Type) {
    about: string;
    version: string;
    #insert -> string {
        return help_members(T);
    };
}

// One bool per field of `T`: true when the user supplied the option (on the command line or
// through its `@env`). A subcommand field holds the Is_Set of that subcommand's struct.
Is_Set :: struct(T: Type) {
    #insert -> string {
        return set_members(T);
    };
}

// Parses `argv` (element 0 is the program name) into a `T`. Never prints and never exits.
// `error.kind` is .NONE on success; .HELP and .VERSION carry the text to print in `error.text`.
// On any error the returned struct holds what was parsed so far. `environment` holds
// "NAME=value" entries that win over the process environment when an `@env` option is looked
// up (for tests; leave it empty to read the real one).
try_parse :: ($T: Type, $help: Help(T), argv: [] string, environment: [] string = .[]) -> T, Error, Is_Set(T) {
    out: T;
    set: Is_Set(T);
    state: Parse_State;
    state.argv = argv;
    state.cursor = 1;
    state.environment = environment;
    state.program = program_name(argv);
    describe_root :: (program: string) -> Command_Spec {
        return describe(T, help, program);
    }
    state.describe = describe_root;
    ok := parse_command(T, help, *state, *out, *set);
    if !ok finish_error(*state);
    return out, state.error, set;
}

// Parses the process's arguments. `--help` and `--version` print and exit 0; a bad command
// line prints the error with a usage hint to stderr and exits 2.
parse :: ($T: Type, $help: Help(T)) -> T, Is_Set(T) {
    argv := get_command_line_arguments();
    out, error, set := try_parse(T, help, argv);
    if error.kind == .HELP || error.kind == .VERSION {
        print("%", error.text);
        exit(0);
    }
    if error.kind != .NONE {
        report(error);
        exit(2);
    }
    return out, set;
}

// The completion script for `shell`, to be saved where that shell loads completions from. The
// program name is argv[0]'s basename unless `program` says otherwise.
completions :: ($T: Type, $help: Help(T), shell: Shell, program := "") -> string {
    spec := describe(T, help, ifx program.count then program else program_name(get_command_line_arguments()));
    return completion_script(*spec, shell);
}

// The help text of the whole program (what `--help` prints), for tools that show it themselves.
help_text :: ($T: Type, $help: Help(T), program := "", color := false) -> string {
    spec := describe(T, help, ifx program.count then program else program_name(get_command_line_arguments()));
    return render_help(*spec, .[], color);
}

#scope_module

#import "Basic";
#import "String";
#import "Compiler";

#load "runtime.jai";
#load "render.jai";
#load "platform.jai";
#load "generate.jai";
#load "completions.jai";

// The generated parser for `T`; recursive for subcommands.
parse_command :: ($T: Type, $help: Help(T), state: *Parse_State, out: *T, set: *Is_Set(T)) -> bool {
    #insert -> string {
        analysis := analyze(T, help.version.count > 0, false);
        if !analysis.ok return "return false;";
        return tprint("%\n%", generate_help_checks(*analysis), generate_command(*analysis, help.version.count > 0));
    };
}

describe :: ($T: Type, $help: Help(T), program: string) -> Command_Spec {
    spec: Command_Spec;
    spec.name = program;
    spec.about = help.about;
    spec.version = help.version;
    defaults: T;
    #insert -> string {
        analysis := analyze(T, help.version.count > 0, true);
        if !analysis.ok return "";
        return generate_description(*analysis);
    };
    return spec;
}

program_name :: (argv: [] string) -> string {
    if !argv.count return "program";
    name := argv[0];
    cut := find_index_of_any_from_right(name, "/\\");
    if cut >= 0 name = slice(name, cut + 1, name.count - cut - 1);
    if ends_with(name, ".exe") name = slice(name, 0, name.count - 4);
    return ifx name.count then name else "program";
}

// Fills in the rest of a failed parse: the help or version text, the usage line, the
// suggestion for a mistyped name.
finish_error :: (state: *Parse_State) {
    error := *state.error;
    root := state.describe(state.program);
    active := find_command(*root, state.path);

    names: [..] string;
    array_add(*names, state.program);
    for state.path array_add(*names, it);
    error.command = join(..names, " ");

    if error.kind == .HELP {
        error.text = render_help(*root, state.path, color_enabled(.OUTPUT));
        return;
    }
    if error.kind == .VERSION {
        error.text = tprint("% %\n", root.name, root.version);
        return;
    }
    error.usage = usage_line(*root, state.path);

    if error.kind == .UNKNOWN_OPTION {
        long: [..] string;
        for active.options if !it.hidden {
            array_add(*long, tprint("--%", it.long));
            if it.negatable array_add(*long, tprint("--no-%", it.long));
        }
        array_add(*long, "--help");
        if root.version.count array_add(*long, "--version");
        if begins_with(error.option, "--") error.suggestion = copy_string(closest(error.option, long));
    } else if error.kind == .UNKNOWN_COMMAND {
        commands: [..] string;
        for active.subcommands array_add(*commands, it.name);
        error.suggestion = copy_string(closest(error.value, commands));
    }
}
