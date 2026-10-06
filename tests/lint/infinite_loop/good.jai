#import "Basic";

running := true;

step :: (i: *int) {
    i.* += 1;
}

sum :: (values: [] int) -> int {
    total := 0;
    i := 0;
    while i < values.count {
        total += values[i];
        i += 1;
    }
    j := 0;
    while j < 3 {
        if total > 100 break;
        total += 1;
    }
    k := 0;
    while k < 3 step(*k);
    while running running = false;
    while true {
        return total;
    }
    return total;
}

main :: () {
    print("%\n", sum(.[1, 2, 3]));
}
