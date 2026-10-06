// rules: asm.14
// #asm division by zero traps; run at compile time, the trap fails the build.
divide :: (n: u64, d: u64) -> u64 {
    hi: u64 = 0;
    lo := n;
    #asm { div hi, lo, d; }
    return lo;
}
QUOTIENT :: #run divide(10, 0);
main :: () {
    x := QUOTIENT;
}
