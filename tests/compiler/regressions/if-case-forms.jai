// if-case statements: no fall-through by default, `#through`, default case, and the types
// that may be switched on.
#import "Basic";

Color :: enum {
    RED;
    GREEN;
    BLUE;
}

pick :: (n: int) -> int {
    if n == {
        case 1; return 10;
        case 2; return 20;
        case; return -1;
    }
}

describe :: (c: Color) -> string {
    if #complete c == {
        case .RED; return "red";
        case .GREEN; return "green";
        case .BLUE; return "blue";
    }
}

with_default :: (c: Color) -> string {
    if #complete c == {
        case .RED; return "r";
        case .GREEN; return "g";
        case .BLUE; return "b";
        case; return "?";
    }
}

grouped :: (n: int) -> int {
    steps := 0;

    if n == {
        case 1; #through;
        case 2; #through;
        case 3;
            steps += 1;
        case 4; #through;
        case;
            steps += 100;
    }

    return steps;
}

word :: (s: string) -> int {
    if s == {
        case "one"; return 1;
        case "two"; return 2;
        case "three"; return 3;
    }

    return 0;
}

kind :: (t: Type) -> int {
    if t == {
        case int; return 1;
        case float; return 2;
        case string; return 3;
    }

    return 0;
}

twice :: (x: int) -> int {
    return x * 2;
}

thrice :: (x: int) -> int {
    return x * 3;
}

pick_proc :: (f: (int) -> int) -> int {
    if f == {
        case twice; return 2;
        case thrice; return 3;
    }

    return 0;
}

main :: () {
    assert(pick(1) == 10);
    assert(pick(2) == 20);
    assert(pick(99) == -1);

    assert(describe(.RED) == "red");
    assert(describe(.BLUE) == "blue");
    assert(with_default(.GREEN) == "g");

    // Cases do not fall through: only the matching block runs.
    ran := 0;
    value := 2;

    if value == {
        case 1; ran += 1;
        case 2; ran += 10;
        case 3; ran += 100;
    }

    assert(ran == 10);

    // #through continues into the next block, and the default can be entered that way.
    assert(grouped(1) == 1);
    assert(grouped(2) == 1);
    assert(grouped(3) == 1);
    assert(grouped(4) == 100);
    assert(grouped(9) == 100);

    // No matching case and no default: nothing runs.
    touched := false;

    if value == {
        case 7; touched = true;
    }

    assert(!touched);

    // Each case is its own scope.
    if value == {
        case 1;
            local := "a";

            assert(local == "a");
        case 2;
            local := 5;

            assert(local == 5);
    }

    assert(word("two") == 2);
    assert(word("three") == 3);
    assert(word("four") == 0);

    assert(kind(int) == 1);
    assert(kind(string) == 3);
    assert(kind(bool) == 0);

    assert(pick_proc(twice) == 2);
    assert(pick_proc(thrice) == 3);

    // Characters and small integer types.
    byte: u8 = #char "x";
    if byte == {
        case #char "x"; ran = 1;
        case #char "y"; ran = 2;
    }

    assert(ran == 1);

    // Switching on a flag value from an expression.
    total := 0;

    for 0..5 {
        if it % 3 == {
            case 0; total += 1;
            case 1; total += 10;
            case 2; total += 100;
        }
    }

    assert(total == 2 * 111);

    print("ok\n");
}
