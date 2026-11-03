#import "Basic";

Shape :: enum {
    CIRCLE;
    SQUARE;
    TRIANGLE;
}

sides :: (kind: Shape) -> int {
    if kind == {
        case .CIRCLE; return 0;
        case .SQUARE; return 4;
    }

    return -1;
}

main :: () {
    print("%\n", sides(.TRIANGLE));
}
