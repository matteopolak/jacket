// Declarations shared by the Args tests (loaded by them; not a test itself).

Args :: #import "Extensions/Args";

Color :: enum {
    AUTO;
    ALWAYS;
    NEVER;
}

// The shape from the design: positionals, flags, a bounded option read from the environment,
// an enum, a metavar and a counter.
Cli :: struct {
    paths: [..] string; @positional
    check: bool; @short=c
    jobs: int = 4; @short=j @env=JAIFMT_JOBS @range(1,64)
    color: Color = .AUTO;
    config: string; @value=FILE
    verbose: u8; @short=v @count
}

HELP :: Args.Help(Cli).{
    about = "Format Jai source files.",
    version = "1.2.3",
    paths = "Files or directories to format",
    check = "Do not write; list the files that would change",
    jobs = "Parallel jobs",
    color = "When to colour the output",
    config = "Use this config file",
    verbose = "Say more (repeat for more)",
};

// Every field type once, with defaults, ranges and repeatable options.
Kinds :: struct {
    flag: bool;
    on_by_default: bool = true;
    small: u8;
    wide: s64;
    signed: s16 = -5;
    ratio: float64 = 0.5;
    single: float;
    name: string;
    color: Color;
    fallback: Color = .NEVER;
    ids: [..] int; @short=i
    tags: [..] string;
    shades: [..] Color;
    tolerance: float; @range(0.0,1.5)
    level: int; @range(-3,3) @long=lvl
    output_dir: string; @short=o
    secret: string; @hidden
}

KINDS_HELP :: Args.Help(Kinds).{
    flag = "A flag", on_by_default = "A flag that starts on", small = "A u8", wide = "An s64", signed = "An s16",
    ratio = "A float64", single = "A float", name = "A string", color = "An enum", fallback = "An enum with a non-zero default", ids = "Repeat for many ids",
    tags = "Repeat for many tags", shades = "Repeat for many colours", tolerance = "Bounded float", level = "Bounded int",
    output_dir = "Where to write",
};

// Relations between options.
Relations :: struct {
    input: string; @short=i @conflicts=stdin
    stdin: bool; @conflicts=input @requires=format
    format: string;
    mode: string; @required
    paths: [..] string; @positional
}

RELATIONS_HELP :: Args.Help(Relations).{ input = "Input file", stdin = "Read standard input", format = "Format", mode = "Mode", paths = "More files" };

// Two required positionals and a trailing list.
Copy :: struct {
    source: string; @positional @required
    target: string; @positional @required @value=DEST
    extra: [..] string; @positional
    force: bool; @short=f
}

COPY_HELP :: Args.Help(Copy).{ source = "What to copy", target = "Where to put it", extra = "Also copy these", force = "Overwrite" };

// Subcommands, with a global option and a nested level.
Command :: enum {
    BUILD;
    RUN;
    REMOTE_ADD;
}

Build_Args :: struct {
    release: bool; @short=r
    target: string; @env=BUILD_TARGET
    files: [..] string; @positional
}

Run_Args :: struct {
    program: string; @positional @required
    arguments: [..] string; @positional
}

Remote_Add_Args :: struct {
    name: string; @positional @required
    url: string; @positional @required
}

Tool :: struct {
    verbose: u8; @short=v @count @global
    config: string; @value=FILE @global
    quiet: bool; @short=q @conflicts=verbose
    command: Command; @subcommand
    build: Build_Args; @subcommand
    run: Run_Args; @subcommand
    remote_add: Remote_Add_Args; @subcommand
}

TOOL_HELP :: Args.Help(Tool).{
    about = "A build tool.",
    version = "0.1.0",
    verbose = "More output (repeat for more)",
    config = "Configuration file",
    quiet = "Less output",
    build = .{ about = "Compile the project", release = "Optimise", target = "Target triple", files = "Files to build" },
    run = .{ about = "Run a program", program = "Program to run", arguments = "Arguments for the program" },
    remote_add = .{ about = "Add a remote", name = "Remote name", url = "Remote URL" },
};

// A selector that may be left out.
Maybe_Command :: enum {
    INIT;
    CLEAN;
}

Init_Args :: struct {
    path: string; @positional
}

Clean_Args :: struct {
    all: bool;
}

Optional_Tool :: struct {
    dry_run: bool; @short=n
    command: Maybe_Command; @subcommand @optional
    init: Init_Args; @subcommand
    clean: Clean_Args; @subcommand
}

OPTIONAL_HELP :: Args.Help(Optional_Tool).{
    dry_run = "Do nothing",
    init = .{ path = "Where" },
    clean = .{ all = "Everything" },
};
