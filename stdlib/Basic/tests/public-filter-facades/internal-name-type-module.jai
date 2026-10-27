// test: warns("`operator -` is not in scope here")
Basic :: #import "Basic";

using,only(.["operator-"]) Basic;

main :: () {
    result := Basic.seconds_to_apollo(7) - Basic.seconds_to_apollo(2);
    // `-` is found in `Apollo_Time`'s module, with a warning.
    Basic.assert(result.low == 5000000000000000);
    Basic.print("ok\n");
}
