// Required source rejection: numeric formatting must not erase nominal typing.
#import "Basic";

Distinct_Number :: #type,distinct float64;
requires_plain_float :: (value: float64) {}

main :: () {
    number := cast(Distinct_Number) cast(float64) 8.5;
    requires_plain_float(number);
}
