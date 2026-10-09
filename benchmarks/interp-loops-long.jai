// Interpreter: the loops of interp-loops with ten times the rounds.
#import "Basic";

main :: () {
    values: [..] int;
    for 0..199_999 array_add(*values, (it * 7919) % 1000);
    total := 0;
    for round: 0..199 {
        for *values {
            it.* = (it.* * 31 + round) % 1009;
            total += it.*;
        }
    }
    print("%\n", total);
}
