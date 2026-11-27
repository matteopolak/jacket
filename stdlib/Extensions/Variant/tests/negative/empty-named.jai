// test: compile_fail("Variant(struct {}): the struct has no fields, so the Variant has no cases")
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(struct {});
}
