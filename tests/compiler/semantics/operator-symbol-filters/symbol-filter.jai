Basic :: #import "Basic";

using,only(.["-"]) Basic;

main :: () {
    a := Basic.seconds_to_apollo(7);
    b := Basic.seconds_to_apollo(2);
    elapsed := a - b;
    Basic.assert(elapsed.low == 5000000000000000 && elapsed.high == 0);
}
