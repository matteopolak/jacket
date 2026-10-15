// Original dependency reduction: Linux compile must accept canonical atomic calls.
// Source-check only: an uninitialized ring must never be submitted at runtime.
Uring :: #import "Extensions/Io_Uring";

main :: () {
    ring: Uring.Ring;
    Uring.submit_and_wait(*ring, 1);
}
