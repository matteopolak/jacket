// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") compile_fail
A :: #import "Android/Toolchain";

main :: () {
    A.JKS_MAGIC = 0;
}
