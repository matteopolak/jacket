// Found by fuzzing: looking up a missing member through a cycle of `using` pointers
// recursed until the stack overflowed.
A :: struct { using b: *B; }
B :: struct { using a: *A; }

main :: () {
    a: A;
    a.missing = 1;
}
