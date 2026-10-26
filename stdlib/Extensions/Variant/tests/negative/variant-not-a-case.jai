// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v := variant(Variant(s64, string), true);
}
