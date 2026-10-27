// test: compile_fail
#import "internal/basic";

main :: () {
    value := portable_float(1.0, 102, 6);
}
