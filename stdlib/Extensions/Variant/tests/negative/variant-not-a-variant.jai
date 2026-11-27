// test: compile_fail("variant: `s64` is not a Variant")
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v := variant(s64, 1);
}
