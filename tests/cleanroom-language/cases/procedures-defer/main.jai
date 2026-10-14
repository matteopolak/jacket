#import "Basic";

weighted_sum :: (left: s64, right: s64 = 4, scale := 2) -> s64 {
    return (left + right) * scale;
}

raise_after :: (value: *s64) {
    defer value.* += 5;
    value.* *= 3;
}

main :: () {
    first := weighted_sum(3);
    second := weighted_sum(3, scale = 3);
    total := 2;
    raise_after(*total);

    assert(first == 14 && second == 21, "defaults and named argument skipping");
    assert(total == 11, "defer runs when the procedure scope exits");
    print("ok\n");
}
