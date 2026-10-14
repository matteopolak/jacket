// Expected: a compile-time rejection, preserving the public constant table.
M :: #import "pl_mpeg";

main :: () {
    M.QUANT_TAB[0].levels = 9;
}
