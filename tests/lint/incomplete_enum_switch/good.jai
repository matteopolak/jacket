#import "Basic";

Shape :: enum {
    CIRCLE;
    SQUARE;
    TRIANGLE;
}

Perm :: enum_flags {
    READ;
    WRITE;
}

Alias :: enum {
    A :: 0;
    B;
    C :: 1;
}

sides :: (kind: Shape) -> int {
    // Every member listed.
    if kind == {
        case .CIRCLE; return 0;
        case .SQUARE; return 4;
        case .TRIANGLE; return 3;
    }

    return -1;
}

complete :: (kind: Shape) -> int {
    if #complete kind == {
        case .CIRCLE; return 0;
        case .SQUARE; return 4;
        case .TRIANGLE; return 3;
    }
}

fallback :: (kind: Shape) -> int {
    if kind == {
        case .CIRCLE; return 0;
        case; return 1;
    }

    return 2;
}

number :: (n: int) -> int {
    if n == {
        case 1; return 10;
        case 2; return 20;
    }

    return 0;
}

flags :: (p: Perm) -> int {
    if p == {
        case .READ; return 1;
    }

    return 0;
}

main :: () {
    print("% % % % %\n", sides(.CIRCLE), complete(.SQUARE), fallback(.CIRCLE), number(1), flags(.READ));
}
