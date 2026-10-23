#import "Basic";

// Scores 26: each line's comment says what it adds.
process :: (items: [] int, limit: int) -> int {
    total := 0;

    for items { // 1
        if it < 0 continue; // 2: an `if` one level deep

        if it > limit && limit > 0 { // 2, and 1 for `&&`
            for 0..it { // 3
                if it % 2 == 0 { // 4
                    total += it;
                } else if it % 3 == 0 { // 1
                    total -= it;
                } else { // 1
                    while total > limit { // 5
                        total /= 2;
                    }
                }
            }
        } else { // 1
            total += ifx it > 10 then 10 else it; // 3
        }
    }

    if total > 100 || total < -100 total = 0; // 1, and 1 for `||`

    return total;
}

main :: () {
    print("%\n", process(.[1, 2, 3], 2));
}
