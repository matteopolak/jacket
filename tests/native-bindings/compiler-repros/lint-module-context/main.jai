// Original module-root lint reproducer. Program compilation and explicit
// program-context lint succeed; inferred module lint freezes Context too early.
#import "Basic";
Example :: #import "Example";

main :: () {
    assert(Example.VALUE);
    assert(context.probe_marker == 0);
}
