// test: skip_on(.WASM) compile_fail
// Reviewed EVP declarations remain implementation-scoped.
#import "Android/Toolchain";

main :: () {
    EVP_PKEY_free(null);
}
