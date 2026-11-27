// test: compile_fail("get_pointer: `u8` is not a case of `Variant(s64, string)`. The cases are .S64 (s64), .STRING (string)")
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(s64, string);

    pointer := get_pointer(*v, u8);
}
