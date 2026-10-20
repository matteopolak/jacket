// Case values may be any constant expression, strings included; a #complete switch needs an enum.
#import "Basic";

TWO :: 2;

Color :: enum {
    RED;
    GREEN;
}

describe :: (n: int) -> string {
    if n == {
        case 1; return "one";
        case TWO; return "two";
        case 1 + 2; return "three";
        case 4; #through;
        case 5; return "four or five";
        case; return "other";
    }

    return "";
}

word :: (s: string) -> int {
    if s == {
        case "a"; return 1;
        case "b"; return 2;
        case; return 0;
    }

    return -1;
}

name :: (c: Color) -> string {
    if #complete c == {
        case .RED; return "red";
        case .GREEN; return "green";
    }

    return "";
}

main :: () {
    assert(describe(1) == "one");
    assert(describe(2) == "two");
    assert(describe(3) == "three");
    assert(describe(4) == "four or five");
    assert(describe(5) == "four or five");
    assert(describe(9) == "other");
    assert(word("a") == 1 && word("b") == 2 && word("c") == 0);
    assert(name(.RED) == "red" && name(.GREEN) == "green");
    print("ok\n");
}
