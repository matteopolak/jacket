// test: compile_fail("set: `bool` is not a case of `Variant(s64, string)`. The cases are .S64 (s64), .STRING (string)")
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(s64, string);

    set(*v, true);
}
