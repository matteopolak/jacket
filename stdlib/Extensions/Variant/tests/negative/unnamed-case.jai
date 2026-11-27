// test: compile_fail("Variant(s64, [..] string): `[..] string` has no name to give its case. Name the cases: `Variant(struct { name: [..] string; ... })`")
#import "Basic";
#import "Extensions/Variant";

// An array type has no name to give its case.
main :: () {
    v: Variant(s64, [..] string);
}
