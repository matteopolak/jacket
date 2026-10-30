// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)") should_fail("Android NDK ABI is unavailable for this CPU")
// Independent unsupported-target diagnostic; no fabricated ABI for custom CPUs.
A :: #import "Android/Toolchain";

main :: () {
    A.get_android_project_jni_architecture_name(.CUSTOM);
}
