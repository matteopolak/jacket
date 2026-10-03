ANDROID_HELPER_VERSION :: 2;
ANDROID_DYNAMIC_LOADER_VERSION :: 3;
ANDROID_LOADER_LIB_NAME :: "loader";

android_get_helper_program_executable_name :: () -> string {
    #if BUILD_OS == .WINDOWS return "jai-android-helper-v2.exe";
    else return "jai-android-helper-v2";
}

android_maybe_compile_helper_program :: (app_name: string, package_name: string, output_path := "", activity := DEFAULT_ACTIVITY, debug := false) -> bool #compile_time {
    return android_host_helper_compile(app_name, package_name, output_path, activity, debug, false);
}

android_compile_helper_program :: (app_name: string, package_name: string, output_path := "", activity := DEFAULT_ACTIVITY, debug := false) -> bool #compile_time {
    return android_host_helper_compile(app_name, package_name, output_path, activity, debug, true);
}

#scope_module
DEFAULT_ACTIVITY :: "android.app.NativeActivity";

#scope_file
#import "Basic";
#load "../../internal/android-toolchain/android-helper.jai";
