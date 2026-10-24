#import "Basic";

Color :: enum {
    COLOR_RED;
    COLOR_GREEN;
    COLOR_BLUE;
}

Token_Kind :: enum u8 {
    TOKEN_KIND_IDENT :: 1;
    TOKEN_KIND_NUMBER;
}

ShapeKind :: enum {
    ShapeKindCircle;
    ShapeKindSquare;
}

main :: () {
    Mode :: enum {
        mode_fast;
        mode_slow;
    }

    print("% % % %\n", Color.COLOR_RED, Token_Kind.TOKEN_KIND_NUMBER, ShapeKind.ShapeKindSquare, Mode.mode_fast);
}
