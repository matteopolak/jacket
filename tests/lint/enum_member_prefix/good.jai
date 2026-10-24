#import "Basic";

Color :: enum {
    RED;
    GREEN;
    BLUE;
}

// One member does not repeat the name.
Token_Kind :: enum u8 {
    TOKEN_KIND_IDENT :: 1;
    NUMBER;
}

// A single member is not a pattern.
Only :: enum {
    ONLY_ONE;
}

// The name alone is no prefix of anything.
Level :: enum {
    LEVEL;
    LEVEL_TWO;
}

// Members that merely share a letter with the name.
Shape :: enum {
    SQUARE;
    SPHERE;
}

main :: () {
    print("% % % % %\n", Color.RED, Token_Kind.NUMBER, Only.ONLY_ONE, Level.LEVEL, Shape.SQUARE);
}
