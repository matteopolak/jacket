// A struct literal inside `type_of` is a literal of its own type, not of the type of the whole
// expression: the language server oracle (docs/tools/lsp-oracle.md) matches literals by their
// closing brace, because `(Outer.{}).first` starts where `Outer.{}` does.
#import "Basic";

Inner :: struct {
    a: s64;
    b: s64;
}

Outer :: struct {
    first: Inner;
}

Probe :: type_of((Outer.{}).first);

main :: () {
    value: Probe;
    assert(value.a == 0 && value.b == 0, "a default Inner");
    print("ok\n");
}
