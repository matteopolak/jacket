// Used by tests/stdlib/compiler-noted-module-procs.jai.

// Nothing calls this, and its body names a procedure that does not exist: a module's
// uncalled procedures are never checked, notes or not.
stale_wrapper :: (x: int) -> int {
    return renamed_long_ago(x);
} @Tagged

bump :: (x: int) -> int {
    return x + 1;
} @Tagged
