// Independent unsupported-target diagnostic; no fabricated ABI for custom CPUs.
A :: #import "Android/Toolchain";

main :: () {
    A.get_android_project_jni_architecture_name(.CUSTOM);
}
