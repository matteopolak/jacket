// Args rejects this declaration at compile time.
#import "Basic";
Args :: #import "Extensions/Args";

Inner :: struct { z: int; }

Cli :: struct {
    a: int; @range(1)
}

main :: () {
    cli := Args.try_parse(Cli, Args.Help(Cli).{}, string.["x"]);
}
