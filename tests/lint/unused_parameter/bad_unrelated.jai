#import "Basic";

// Not loaded by the cases: an unrelated procedure that happens to share the name `scale` with
// the one in `bad.jai`. Its parameters differ, so it is no variant and `label` is still reported.
scale :: (amount: float) -> float {
    return amount * 2;
}

main :: () {
    print("%\n", scale(1.5));
}
