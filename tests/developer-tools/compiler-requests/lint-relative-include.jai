#import "Basic";
Probe :: #import "relative-include";

main :: () {
    assert(Probe.original_value() == 42);
}
