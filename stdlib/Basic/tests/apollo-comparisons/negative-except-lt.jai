B :: #import "Basic";

using,except(.["<"]) B;

main :: () {
    a := B.seconds_to_apollo(1);
    b := B.seconds_to_apollo(2);
    rejected := a < b;
}
