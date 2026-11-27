// test: compile_fail("variant: `Variant(s64, string)` has no `void` case. Add one: `Variant(void, ...)`, or a field `none: void;` in the named form. The cases are .S64 (s64), .STRING (string)")
#import "Basic";
#import "Extensions/Variant";

// `variant(V)` selects the `void` case, and this Variant has none.
main :: () {
    v := variant(Variant(s64, string));
}
