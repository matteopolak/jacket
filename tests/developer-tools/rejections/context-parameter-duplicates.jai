// Reviewed language contract: repeated context field names must reject.
A :: #import "context-parameter-module"(VALUE = 1);
B :: #import "context-parameter-module"(VALUE = 2);

main :: () {
    A.set_value();
    B.set_value();
}
