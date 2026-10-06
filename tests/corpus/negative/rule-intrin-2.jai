// rules: intrin.2
// An #intrinsic whose name the compiler does not know is rejected at the call.
frob :: (x: int) -> int #intrinsic;
main :: () { y := frob(1); }
