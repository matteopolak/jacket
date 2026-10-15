#import "Basic";
AT :: #import "Android/Toolchain";

main :: () {
    compile := AT.android_compile_helper_program;
    maybe_compile := AT.android_maybe_compile_helper_program;

    assert(compile != null && maybe_compile != null);
    assert(AT.ANDROID_HELPER_VERSION == 2);
    assert(AT.ANDROID_DYNAMIC_LOADER_VERSION == 3);
    assert(AT.ANDROID_LOADER_LIB_NAME == "loader");
    assert(AT.android_get_helper_program_executable_name().count > 0);
}
