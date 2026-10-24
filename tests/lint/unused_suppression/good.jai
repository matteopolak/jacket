#import "Basic";

// jailint: allow-file(todo_marker)

scale :: (a: int) -> int {
    // jailint: allow(unused_variable)
    spare := 4;

    // A name that is not a rule is for `unknown_allow` to report.
    // jailint: allow(no_such_rule)
    total := a * 2;

    return total;
}

shift :: (a: int, b: int) -> int {
    scratch := 9;
    return a + b;
} @jailint_allow(unused_variable)

// TODO: fold this into `scale`.
fold :: (a: int) -> int {
    return a;
}

platform :: (a: int) -> int {
    #if OS == .WINDOWS {
        // jailint: allow(unused_variable)
        extra := 1;
    }

    return a;
}

main :: () {
    print("%\n", scale(2) + shift(3, 0) + fold(1) + platform(1));
}
