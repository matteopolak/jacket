// Held to every system by the test settings: Windows has no POSIX.
P :: #import "POSIX";

main :: () {
    P.getpid();
}
