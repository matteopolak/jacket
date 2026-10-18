// Compile-time execution: hot loops, calls and a table lookup inside `#run`.
#import "Basic";

mix :: (state: u32, value: int) -> u32 {
    state ^= cast(u32) value;
    state *= 16777619;
    return state ^ (state >> 13);
}

churn :: (rounds: int) -> u32 {
    state: u32 = 2166136261;
    for 0..rounds - 1 state = mix(state, it);
    return state;
}

collatz :: (limit: int) -> int {
    longest := 0;

    for start: 1..limit {
        n := start;
        steps := 0;

        while n != 1 {
            n = ifx n & 1 then 3 * n + 1 else n / 2;
            steps += 1;
        }

        if steps > longest longest = steps;
    }

    return longest;
}

FIRST :: #run churn(12_000_000);
SECOND :: #run collatz(150_000);

main :: () {
    print("% %\n", FIRST, SECOND);
}
