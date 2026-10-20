// A `#char` literal and a `type_info(T)` are constants as `case` values even when the
// switched value is an enum or a type pointer (jai_parser's lexer, toml-jai's serializer).
#import "Basic";

Token :: struct {
    Kind :: enum {
        IDENTIFIER :: 256;
        NUMBER;
    }
    kind: Kind;
}

A :: struct {
    x: int;
}

B :: struct {
    y: float;
}

classify :: (kind: Token.Kind) -> int {
    if kind == {
        case #char "+"; return 1;
        case #char "-"; return 2;
        case .IDENTIFIER; return 3;
        case; return 0;
    }

    return -1;
}

which :: (info: *Type_Info_Struct) -> int {
    if info == {
        case type_info(A); return 1;
        case type_info(B); return 2;
    }

    return 0;
}

main :: () {
    assert(classify(cast(Token.Kind) #char "+") == 1);
    assert(classify(cast(Token.Kind) #char "-") == 2);
    assert(classify(.IDENTIFIER) == 3);
    assert(classify(.NUMBER) == 0);
    assert(which(cast(*Type_Info_Struct) type_info(A)) == 1);
    assert(which(cast(*Type_Info_Struct) type_info(B)) == 2);
    assert(which(null) == 0);
    print("ok\n");
}
