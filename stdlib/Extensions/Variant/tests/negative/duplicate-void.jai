// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// In a type list a `void` case is named `none`, so it may appear once.
main :: () {
    v: Variant(void, s64, void);
}
