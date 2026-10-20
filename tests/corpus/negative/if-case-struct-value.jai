#import "Basic";

Point :: struct {
    x: int;
}

main :: () {
    p: Point;
    if p == {
        case .{1}; print("one\n");
    }
}
