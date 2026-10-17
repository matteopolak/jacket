// The playground examples: whitespace in the input does not change the output.
main :: () {
    a: int, b: int;
    arr := int.[1, 2, 3];
    for arr {
        print("%\n", it);
    }
    x := 1;
    if x > 0 {
        print("hi %\n", x);
    }
    s := "a";
    if s == "a" {
        print("string header\n");
    }
    if x == {
        case 1; print("one\n");
        case 2;
            a = 1;
            b = 2;
        case; #through;
    }
    if x == 1 {
        a = 2;
    } else {
        a = 3;
    }
    if x == 2 {
        a = 4;
    } else if x == 3 {
        a = 5;
    } else {
        a = 6;
    }
    if a {}
    if b {}
    {
        inner := 1;
    }
    after := 2;
    n := x * 2 + 1 - x / 3;
    m := (n << 2) | (n >> 1);
    for i: 0..n - 1 {}
    for < i: 0..n {}
    v := Vec.{ x = 1, y = 2 };
    e := Vec.{};
    p := *x;
    neg := -n;
    q := cast(*u8) p;
    w := xx (n + 1);
    t: [4] int;
    d: [..] *u8;
    k: int : 5;
    if (a) return;
    f := () -> int {
        return 1;
    };
    call(a, b, c); // trailing comment
    z := 1; /* block */
}

Point :: struct {
    x: float;
    y: float;
}

Kind :: enum u8 {
    A;
    B;
    C :: 5;
}

poly :: (x: $T/Interface, y: [] $U) {}

short :: () {
    return;
}

#if OS == .WINDOWS {
    #load "windows.jai";
}
