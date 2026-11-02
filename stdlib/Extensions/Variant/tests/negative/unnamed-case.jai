// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// An array type has no name to give its case.
main :: () {
    v: Variant(s64, [..] string);
}
