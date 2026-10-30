// A statement starting with `ifx` is an `if`: it may switch on a value, and its assigning
// branches may be parenthesized.
// rules: flow.30 flow.31
#import "Basic";

Mode :: enum {
    A;
    B;
    C;
}

main :: () {
    mode := Mode.B;
    chosen := 0;

    ifx mode == {
        case .A;
            {
                chosen = 1;
            }
        case .B;
            {
                chosen = 2;
            }
        case;
            chosen = 3;
    }

    assert(chosen == 2);

    t := Mode.A;
    ifx (t == .A) then (t = .B) else t = .C;
    assert(t == .B);
    ifx (t == .A) then (t = .B) else (t = .C);
    assert(t == .C);

    n := 1;
    if n == 1 ifx (t == .C) then (n = 5) else n += 1;

    assert(n == 5);
    print("ok\n");
}
