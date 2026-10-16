// Independent reduction of target-typed decimal output; compatibility oracle pending.
#import "Basic";

main :: () {
    value := cast(S128) (-42);
    assert(value.low == 0xffffffffffffffd6 && value.high == -1);
}
