#import "Basic";

main :: () {
    x := 3;
    if x == {
        case 1; print("one\n");
        case 2; print("two\n");
        case 1; print("again\n");
    }
}
