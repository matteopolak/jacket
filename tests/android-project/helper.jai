#import "Basic";
AT :: #import "Android/Toolchain";
F :: #import "File_Utilities";
R :: #import "File";
S :: #import "String";

HELPER_BUILT :: #run AT.android_compile_helper_program("Host helper test", "jai.host.fixture", "generated-helper");
HELPER_CACHED :: #run AT.android_maybe_compile_helper_program("Host helper test", "jai.host.fixture", "generated-helper");
INVALID_PACKAGE :: #run AT.android_compile_helper_program("Invalid package", "x;id", "generated-invalid-helper");
INVALID_ACTIVITY :: #run AT.android_compile_helper_program("Invalid activity", "jai.fixture", "generated-invalid-helper", "NativeActivity;id");
INNER_ACTIVITY :: #run AT.android_compile_helper_program("Inner activity", "jai.fixture", "generated-inner-helper", "NativeActivity$Inner");

main :: () {
    assert(HELPER_BUILT && HELPER_CACHED);
    assert(!INVALID_PACKAGE && !INVALID_ACTIVITY);
    assert(INNER_ACTIVITY);
    source, read := R.read_entire_file("generated-inner-helper/.jai-android-helper-v2.jai");

    defer free(source);
    assert(read && S.contains(source, "'jai.fixture/NativeActivity$Inner'"));
    assert(AT.ANDROID_HELPER_VERSION == 2);
    assert(F.file_exists(sprint("generated-helper/%", AT.android_get_helper_program_executable_name())));
    print("Android host launcher helper compilation passed; no device launched\n");
}
