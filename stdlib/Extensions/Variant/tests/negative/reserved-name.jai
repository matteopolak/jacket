// test: compile_fail("Variant(struct { which: s64; other: string; }): a case cannot be named `which`, the Variant uses that name itself")
#import "Basic";
#import "Extensions/Variant";

// `which` is the Variant's own tag field.
main :: () {
    v: Variant(struct {
        which: s64;
        other: string;
    });
}
