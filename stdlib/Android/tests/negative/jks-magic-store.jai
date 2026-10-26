// test: skip_on(.WASM) compile_fail
A :: #import "Android/Toolchain";

main :: () {
    A.JKS_MAGIC = 0;
}
