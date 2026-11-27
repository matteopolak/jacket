// test: compile_fail("Variant needs at least one case: `Variant(Leaf, *Branch)` or `Variant(struct { email: string; phone: string; })`")
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant();
}
