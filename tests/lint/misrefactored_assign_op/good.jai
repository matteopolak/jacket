#import "Basic";

bump :: () -> int {
    counter += 1;
    return counter;
}

counter: int;

main :: () {
    total := 0;
    total += 5;
    other := 3;

    total += other + 2; // the target is not repeated
    total += total * 2; // a different operator
    total -= 10 - total; // `-` does not commute
    total *= other + total; // not the same operator
    counter += bump() + counter; // a call changes `counter` in between
    f := 1.5;
    f -= 2.0 - f; // `-` does not commute
    f += f * 0.5;
    print("% % %\n", total, counter, f);
}
