// test: compile_fail
R :: #import "Runtime_Support";

main :: () {
    R.__jaic_check_failed(0, 0, 0, true, 0, null);
}
