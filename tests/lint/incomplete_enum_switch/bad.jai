#import "Basic";

Shape :: enum {
    CIRCLE;
    SQUARE;
    TRIANGLE;
    HEXAGON;
}

sides :: (kind: Shape) -> int {
    if kind == {
        case .CIRCLE; return 0;
        case .SQUARE; return 4;
    }

    return -1;
}

name_of :: (kind: Shape) -> string {
    result := "";

    if kind == {
        case .CIRCLE; result = "circle";
        case Shape.SQUARE; result = "square";
        case .TRIANGLE; result = "triangle";
    }

    return result;
}

main :: () {
    print("% %\n", sides(.CIRCLE), name_of(.SQUARE));
}
