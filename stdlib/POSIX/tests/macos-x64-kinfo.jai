// Compile-only producer dependency reproducer: macOS x64 POSIX p_realtimer.
P :: #import "POSIX";

main :: () {
    #assert size_of(P.kinfo_proc) > 0;
}
