B :: #import "Basic";

using,except(.["<"]) B;

main :: () {
    a := B.seconds_to_apollo(1);
    b := B.seconds_to_apollo(2);

    // `<` is out of scope here, so it is found in `Apollo_Time`'s module with a warning.
    B.assert(a < b);
    B.print("ok\n");
}
