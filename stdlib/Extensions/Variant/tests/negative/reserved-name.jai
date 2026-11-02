// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

// `which` is the Variant's own tag field.
main :: () {
    v: Variant(struct {
        which: s64;
        other: string;
    });
}
