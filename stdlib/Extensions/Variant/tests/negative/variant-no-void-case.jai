// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// `variant(V)` selects the `void` case, and this Variant has none.
main :: () {
    v := variant(Variant(s64, string));
}
