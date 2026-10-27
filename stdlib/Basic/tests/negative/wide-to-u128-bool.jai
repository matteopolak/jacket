// test: compile_fail
#import "Basic";

main :: () {
    value := to_u128(true);
    print("%", value);
}
