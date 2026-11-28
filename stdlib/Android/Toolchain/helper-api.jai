ANDROID_HELPER_VERSION :: 2;
ANDROID_DYNAMIC_LOADER_VERSION :: 3;
ANDROID_LOADER_LIB_NAME :: "loader";

// Returns the file name of the host helper program that starts the app: `jai-android-helper-v2`, with `.exe`
// on Windows.
android_get_helper_program_executable_name :: () -> string {
    #if BUILD_OS == .WINDOWS return "jai-android-helper-v2.exe";
    else return "jai-android-helper-v2";
}

// Compiles the host helper program unless an up-to-date one exists in `output_path`.
//
// The program starts `package_name/activity` on the device through `adb`. It is reused when its executable
// exists and the source written last time is identical. See [android_compile_helper_program]. Returns `false`
// when `package_name` or `activity` is not a valid name or compilation fails.
android_maybe_compile_helper_program :: (app_name: string, package_name: string, output_path := "", activity := DEFAULT_ACTIVITY, debug := false) -> bool #compile_time {
    return android_host_helper_compile(app_name, package_name, output_path, activity, debug, false);
}

// Compiles the host helper program into `output_path`, replacing any earlier one.
//
// `output_path` defaults to the current folder. The program starts `package_name/activity` on the device
// through `adb`. The compiler is the one named by the `JAIC` environment variable, or `jaic` from the `PATH`;
// `debug` compiles with `-O0`, else `-O2`. Returns `false` when `package_name` or `activity` is not a valid
// name or compilation fails.
android_compile_helper_program :: (app_name: string, package_name: string, output_path := "", activity := DEFAULT_ACTIVITY, debug := false) -> bool #compile_time {
    return android_host_helper_compile(app_name, package_name, output_path, activity, debug, true);
}

#scope_module
DEFAULT_ACTIVITY :: "android.app.NativeActivity";

#scope_file
#import "Basic";
#load "../../internal/android-toolchain/android-helper.jai";
