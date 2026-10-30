// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") compile_fail
// Reviewed EVP declarations remain implementation-scoped.
#import "Android/Toolchain";

main :: () {
    EVP_PKEY_free(null);
}
