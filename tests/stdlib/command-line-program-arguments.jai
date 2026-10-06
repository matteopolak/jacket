// Command_Line.parse_arguments reads the program's real arguments. The runner passes none, so
// every option keeps its default and nothing is marked as set.
#import "Basic";
#import "Command_Line";

Options :: struct {
    verbose: bool;
    level: int = 3;
    name: string = "default";
}

main :: () {
    argv := get_command_line_arguments();
    // Compiled programs see their name as argument zero; `jaic run` without `--` passes none.
    ok, options, is_set, free_arguments := parse_arguments(Options);
    assert(ok, "no arguments parse");
    assert(!options.verbose && options.level == 3 && options.name == "default", "defaults kept: %", options);
    assert(!is_set.verbose && !is_set.level && !is_set.name);
    assert(free_arguments.count == max(argv.count - 1, 0), "% free arguments for % arguments", free_arguments.count, argv.count);

    // The same parse through the explicit entry point, with a program name of our own.
    explicit := string.["program", "-level", "9", "-name", "ünïcode", "-verbose", "extra"];
    ok, options, is_set, free_arguments = parse_arguments_from(explicit, Options);
    assert(ok && options.level == 9 && options.name == "ünïcode" && options.verbose && is_set.level && is_set.name && is_set.verbose);
    assert(free_arguments.count == 1 && free_arguments[0] == "extra");
    // Free arguments refused.
    ok = parse_arguments_from(explicit, Options, flags = .ALIGN_HELP);
    assert(!ok);
    print("ok\n");
}
