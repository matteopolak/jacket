// Args error and help results: the kind, the message, a suggested name, and the help and version text.
#import "Basic";
#import "String";
Args :: #import "Extensions/Args";

Cli :: struct {
    paths: [..] string; @positional
    check: bool; @short=c
    jobs: int = 4; @short=j @range(1,64)
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

run :: (argv: [] string) {
    _, error, set := Args.try_parse(Cli, HELP, argv);
    print("% -> kind=% message=[%] option=[%] value=[%] suggestion=[%] set_jobs=%\n", argv[1], error.kind, error.message, error.option, error.value, error.suggestion, set.jobs);
}

main :: () {
    Args.set_color_mode(.NEVER);

    run(.["tool", "--chek"]);
    run(.["tool", "--jobs"]);
    run(.["tool", "--jobs=100"]);
    run(.["tool", "--jobs=many"]);
    run(.["tool", "--check=yes"]);
    run(.["tool", "-z"]);

    cli, error, set := Args.try_parse(Cli, HELP, .["tool", "--help"]);

    print("help kind=% text_has_about=% text_has_check=%\n", error.kind, contains(error.text, "Does things."), contains(error.text, "--[no-]check"));
    print("%", error.text);
    assert(error.kind == .HELP && contains(error.text, "Does things."));

    cli, error, set = Args.try_parse(Cli, HELP, .["tool", "--version"]);
    print("version kind=% text=%\n", error.kind, error.text);
    print("version text bytes=%\n", error.text.count);
    assert(error.kind == .VERSION && contains(error.text, "tool 2.1"));
    print("ok\n");
}
