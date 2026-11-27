// test: compile_fail("Variant(s64, string, s64): two cases would both be tagged .S64. Name the cases yourself: `Variant(struct { a: s64; b: s64; })`")
#import "Basic";
#import "Extensions/Variant";

// A type list names each case after its type, so a type may appear once; the named form allows more.
main :: () {
    v: Variant(s64, string, s64);
}
