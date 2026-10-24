#import "Basic";

// jailint: allow-file(todo_marker)

scale :: (a: int) -> int {
    // jailint: allow(unused_variable)
    spare := 4;

    // jailint: allow(unused_variable, float_equality)
    leftover := 2;

    total := a * 2; // jailint: allow(float_equality)

    return total;
}

shift :: (a: int) -> int {
    return a + 1;
} @jailint_allow(unused_variable)

main :: () {
    print("%\n", scale(2) + shift(3));
}
