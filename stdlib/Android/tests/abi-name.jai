// Reviewed Android/Toolchain signature; official Android NDK ABI names.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    x64 := A.get_android_project_jni_architecture_name(.X64);
    arm64 := A.get_android_project_jni_architecture_name(.ARM64);

    assert(x64 == "x86_64" && arm64 == "arm64-v8a");
    assert(x64.data != null && arm64.data != null);
    #assert !#exists(A.Impl) && !#exists(A.B);
    print("ok\n");
}
