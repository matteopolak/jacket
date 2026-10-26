// test: skip_on(.WASM) compile_fail
A :: #import "Android/Toolchain";

main :: () {
    A.OID_SUN_JKS_ENCRYPTION.parts[0] = 9;
}
