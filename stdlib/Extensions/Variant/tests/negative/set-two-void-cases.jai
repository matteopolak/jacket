// test: compile_fail("set: `Variant(struct { missing: void; unknown: void; })` has more than one `void` case (.MISSING (void), .UNKNOWN (void)). Set the case by name instead: `v.which = .MISSING;`")
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
