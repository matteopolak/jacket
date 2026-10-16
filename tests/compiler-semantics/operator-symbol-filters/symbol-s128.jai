Basic :: #import "Basic";

using,only(.["-"]) Basic;

main :: () {
    result := Basic.S128.{ 7, 0 } - Basic.S128.{ 2, 0 };
    Basic.assert(result.low == 5 && result.high == 0);
}
