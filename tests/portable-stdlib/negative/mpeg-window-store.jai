// Expected: a compile-time rejection, as for unqualified constant-array stores.
M :: #import "pl_mpeg";

main :: () {
    M.SYNTHESIS_WINDOW[0] = 1.0;
}
