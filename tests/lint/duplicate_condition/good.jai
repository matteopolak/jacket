#import "Basic";

counter := 0;

next :: () -> int {
    counter += 1;
    return counter;
}

describe :: (key: int) {
    if key == 1 print("one\n");
    else if key == 2 print("two\n");
    else print("many\n");

    // Calls may give a different answer each time.
    if next() == 2 print("a\n");
    else if next() == 2 print("b\n");

    if key == {
        case 1; print("1\n");
        case 2; print("2\n");
        case; print("?\n");
    }
}

main :: () {
    describe(1);
}
