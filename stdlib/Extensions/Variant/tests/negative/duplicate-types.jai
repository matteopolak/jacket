// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// A type list names each case after its type, so a type may appear once; the named form allows more.
main :: () {
    v: Variant(s64, string, s64);
}
