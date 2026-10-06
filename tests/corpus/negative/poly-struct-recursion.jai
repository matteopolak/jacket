// Found by fuzzing: each instance contains the next one by value, until the stack overflowed.
P :: struct (T: Type) {
    p: P(*T);
}

main :: () {
    x: P(int);
}
