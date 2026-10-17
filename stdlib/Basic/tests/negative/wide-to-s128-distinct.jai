#import "Basic";

Identifier :: #type,distinct u64;

main :: () {
    value := to_s128(cast(Identifier) 1);
    print("%", value);
}
