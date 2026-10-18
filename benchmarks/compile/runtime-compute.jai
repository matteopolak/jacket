// Generated-code speed: numeric loops, a sieve, hash-table traffic and a sort. Prints a checksum.
#import "Basic";
#import "Math";
#import "Hash_Table";
#import "Sort";

Body :: struct {
    x, y, vx, vy: float64;
    mass: float64;
}

simulate :: (steps: int) -> float64 {
    bodies: [16] Body;
    for *bodies {
        it.x = cast(float64) it_index;
        it.y = cast(float64) (it_index * it_index % 7);
        it.mass = 1.0 + cast(float64) (it_index % 3);
    }

    for 1..steps {
        for *a: bodies {
            for b: bodies {
                dx := b.x - a.x;
                dy := b.y - a.y;
                distance := dx * dx + dy * dy + 0.01;

                a.vx += dx * b.mass / (distance * 100.0);
                a.vy += dy * b.mass / (distance * 100.0);
            }
        }

        for *bodies {
            it.x += it.vx;
            it.y += it.vy;
        }
    }

    energy: float64;
    for bodies energy += 0.5 * it.mass * (it.vx * it.vx + it.vy * it.vy);

    return energy;
}

sieve :: (limit: int) -> int {
    composite := NewArray(limit + 1, bool);

    defer array_free(composite);
    count := 0;

    for 2..limit {
        if composite[it] continue;

        count += 1;
        multiple := it * it;
        while multiple <= limit {
            composite[multiple] = true;
            multiple += it;
        }
    }

    return count;
}

table_traffic :: (count: int) -> int {
    table: Table(int, int);

    defer deinit(*table);
    for 0..count - 1 table_set(*table, it * 7919 % 100003, it);

    hits := 0;

    for 0..count - 1 {
        value := table_find_pointer(*table, it);
        if value hits += value.*;
    }

    return hits;
}

sort_numbers :: (count: int) -> int {
    numbers := NewArray(count, int);

    defer array_free(numbers);
    state: u32 = 12345;

    for 0..count - 1 {
        state = state * 1664525 + 1013904223;
        numbers[it] = cast(int) (state >> 8);
    }

    quick_sort(numbers, (a, b) => a - b);

    return numbers[count / 2];
}

main :: () {
    print("% % % %\n", cast(int) (simulate(40_000) * 1000.0), sieve(8_000_000), table_traffic(1_000_000), sort_numbers(1_500_000));
}
