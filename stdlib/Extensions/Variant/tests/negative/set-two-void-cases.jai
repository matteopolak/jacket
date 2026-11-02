// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// With two `void` cases, `set(*v)` cannot tell which one is meant.
main :: () {
    v: Variant(struct {
        missing: void;
        unknown: void;
    });

    set(*v);
}
