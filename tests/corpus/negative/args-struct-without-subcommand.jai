// Args rejects this declaration at compile time.
#import "Basic";
Args :: #import "Extensions/Args";

Inner :: struct { z: int; }

Cli :: struct {
    a: Inner;
}

main :: () {
    cli := Args.try_parse(Cli, Args.Help(Cli).{}, string.["x"]);
}
