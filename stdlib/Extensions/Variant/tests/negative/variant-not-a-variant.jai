// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v := variant(s64, 1);
}
