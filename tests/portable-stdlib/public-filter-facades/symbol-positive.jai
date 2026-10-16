Basic :: #import "Basic";

using,only(.["-"]) Basic;

main :: () {
    result := Basic.seconds_to_apollo(7) - Basic.seconds_to_apollo(2);
    Basic.assert(result.low == 5000000000000000 && result.high == 0);
    Basic.print("ok\n");
}
