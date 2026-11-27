// test: compile_fail("Variant(void, s64, void): two cases would both be tagged .NONE. Name the cases yourself: `Variant(struct { a: void; b: void; })`")
#import "Basic";
#import "Extensions/Variant";

// In a type list a `void` case is named `none`, so it may appear once.
main :: () {
    v: Variant(void, s64, void);
}
