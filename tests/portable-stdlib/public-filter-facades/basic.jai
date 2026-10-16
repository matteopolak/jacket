#import "Basic";
B :: #import "Basic";
Again :: #import "Basic";

main :: () {
    a := seconds_to_apollo(7);
    b := seconds_to_apollo(2);
    result := a - b;

    assert(type_of(result) == Apollo_Time && result.low == 5000000000000000 && result.high == 0);
    sum := a + b;

    assert(sum.low == 9000000000000000 && sum.high == 0);
    unsigned := U128.{ 0, 1 } - U128.{ 1, 0 };

    assert(unsigned.low == 0xffffffffffffffff && unsigned.high == 0);
    first: (S128, S128) -> S128 = B.operator -;
    second: (Again.S128, Again.S128) -> Again.S128 = Again.operator -;

    assert(first == second);
    assert(type_info(B.Apollo_Time) == type_info(Again.Apollo_Time));
    print("ok\n");
}
