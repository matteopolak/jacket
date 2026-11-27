// test: compile_fail("variant: `bool` is not a case of `Variant(s64, string)`. The cases are .S64 (s64), .STRING (string)")
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v := variant(Variant(s64, string), true);
}
