#import "Basic";

main :: () {
    value := to_u128(cast(s8) -1);
    assert(value.low == 0);
}
