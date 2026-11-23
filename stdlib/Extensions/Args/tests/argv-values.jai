// Args parsing of flags, attached and separate values, environment defaults, positionals and the -- separator.
#import "Basic";
Args :: #import "Extensions/Args";

Cli :: struct {
    paths: [..] string; @positional
    check: bool; @short=c
    jobs: int = 4; @short=j @env=TOOL_JOBS @range(1,64)
    name: string;
}

HELP :: Args.Help(Cli).{
    about = "Does things.",
    version = "2.1",
    paths = "Files to read",
    check = "Only check",
    jobs = "Parallel jobs",
    name = "A label",
};

run :: (label: string, argv: [] string, environment: [] string = .[]) {
    cli, error, set := Args.try_parse(Cli, HELP, argv, environment);
    print("% kind=% check=% jobs=% name=[%] paths=% set_jobs=% set_check=% set_paths=%\n", label, error.kind, cli.check, cli.jobs, cli.name, cli.paths, set.jobs, set.check, set.paths);
}

main :: () {
    Args.set_color_mode(.NEVER);

    run("short and attached", .["tool", "-c", "--jobs=8", "a.jai", "b.jai"]);
    run("separate value", .["tool", "--jobs", "12", "--name", "x y", "file"]);
    run("short value", .["tool", "-j", "3", "-c"]);
    run("dashes after separator", .["tool", "--", "--check", "-x", "-"]);
    run("option before separator", .["tool", "-c", "--", "x"]);
    run("no arguments", .["tool"]);
    run("environment default", .["tool"], .["TOOL_JOBS=16"]);
    run("command line beats environment", .["tool", "--jobs=2"], .["TOOL_JOBS=16"]);
    run("positional interleaved", .["tool", "a", "-c", "b"]);
    print("ok\n");
}
