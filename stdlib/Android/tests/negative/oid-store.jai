// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") compile_fail
A :: #import "Android/Toolchain";

main :: () {
    A.OID_SUN_JKS_ENCRYPTION.parts[0] = 9;
}
