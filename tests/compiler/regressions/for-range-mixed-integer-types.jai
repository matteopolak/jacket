// The two ends of a `for a..b` may have different integer types; the loop counts in the one
// that holds both.
#import "Basic";

main :: () {
    a: u32 = 1;
    b: s64 = 4;
    total := 0;

    for a..b total = total * 10 + it;

    assert(total == 1234, "%", total);

    small: u8 = 2;
    wide: s32 = 5;
    count := 0;

    for small..wide count += 1;

    assert(count == 4, "%", count);

    last := 0;
    for < small..wide last = it;

    assert(last == 2, "%", last);
    print("ok\n");
}
