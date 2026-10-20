// A hash table written here (open addressing, linear probing), so no library code is timed.
// Prints the number of hits and their sum.
#import "Basic";

Slot :: struct {
    key: int;
    value: int;
    used: bool;
}

mix :: (key: int) -> u64 {
    h := cast,no_check(u64) key;
    h ^= h >> 33;
    h *= 0xff51afd7ed558ccd;
    h ^= h >> 33;

    return h;
}

main :: () {
    CAPACITY :: 1 << 22;
    slots := NewArray(CAPACITY, Slot);

    for 0..2_000_000 - 1 {
        key := it * 7919 % 4_000_037;
        index := mix(key) & (CAPACITY - 1);
        while slots[index].used && slots[index].key != key index = (index + 1) & (CAPACITY - 1);
        slots[index] = .{ key, it, true };
    }

    hits, sum := 0, 0;

    for 0..4_000_000 - 1 {
        index := mix(it) & (CAPACITY - 1);
        while slots[index].used {
            if slots[index].key == it {
                hits += 1;
                sum += slots[index].value;
                break;
            }

            index = (index + 1) & (CAPACITY - 1);
        }
    }

    print("% %\n", hits, sum);
}
