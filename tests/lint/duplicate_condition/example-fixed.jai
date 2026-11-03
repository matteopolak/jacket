#import "Basic";

describe :: (key: int) {
    if key == 1 print("one\n");
    else if key == 2 print("two\n");
    else if key == 3 print("three\n");
}

main :: () {
    describe(1);
}
