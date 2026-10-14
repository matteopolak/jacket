// Parse an explicit argument vector and distinguish defaults from explicit values.
#import "Basic";
Command_Line :: #import "Command_Line";

Options :: struct {
    count: s64 = 10;
    enabled: bool = false;
    label: string = "initial";
}

main :: () {
    ok, options, is_set, free_args := Command_Line.parse_arguments(Options,
        args=.["-count", "10", "-enabled", "-label", "cedar", "loose"]);
    assert(ok);
    assert(options.count == 10 && options.enabled && options.label == "cedar");
    assert(is_set.count && is_set.enabled && is_set.label);
    assert(free_args.count == 1 && free_args[0] == "loose");

    ok, options, is_set, free_args = Command_Line.parse_arguments(Options, args=.["-label", "initial"]);
    assert(ok && options.count == 10 && !options.enabled && options.label == "initial");
    assert(!is_set.count && !is_set.enabled && is_set.label);
    assert(free_args.count == 0);
    print("ok\n");
}
