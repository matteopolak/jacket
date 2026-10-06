#import "Basic";

main :: () {
    total := 0;
    total += 5;
    other := 3;
    total = other + total * 2; // not a plain update
    total = 10 - total; // `-` does not commute
    small: u8 = 1;
    small = cast(u8) (small + 1);
    print("% %\n", total, small);
}
