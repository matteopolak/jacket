// Reviewed hooks have empty default bodies; no registration API is implied.
R :: #import "Runtime_Support";

main :: () {
    R.__instrumentation_first();
    R.__instrumentation_second();
    write_string("ok\n");
}
