// Builds the native loader library for the project with the NDK, unless it is current.
//
// The loader is a small shared library that opens the app's real library from the app's external storage
// folder at start-up, so the payload can be replaced without reinstalling the APK. Writes
// `.build_android_loader/libloader.so` under `project_config.project_path`. Returns `true` when the library
// exists and was built from the current source, or `force_rebuild_loader` rebuilt it. Returns `false` when the
// CPU targets of the two option structs differ, `lib_name` is empty, or the NDK or compiler fails.
android_build_or_update_dynamic_loader :: (project_config: Android_Project_Config, build_options: Build_Options, android_build_options: Android_Build_Options, force_rebuild_loader := false) -> bool {
    return android_dynamic_loader_build(project_config, build_options, android_build_options, force_rebuild_loader);
}

// Builds the loader APK, installs it on the device and copies the compiled program onto the device.
//
// Builds the loader with [android_build_or_update_dynamic_loader], generates the project, builds the APK with
// [android_generate_apk] and installs it, keeping app data. The payload copied afterward is
// `build_options.output_path/build_options.output_executable_name`, placed in the app's external storage
// folder. Failures are logged as errors; nothing is returned.
android_build_and_install_loader_app :: (project_config: Android_Project_Config, build_options: Build_Options, android_build_options: Android_Build_Options) {
    if !android_build_or_update_dynamic_loader(project_config, build_options, android_build_options) {
        log_error("Android loader compilation failed\n");
        return;
    }

    config := project_config;
    config.lib_name = ANDROID_LOADER_LIB_NAME;
    options := android_build_options;
    library := android_dynamic_loader_path(project_config);

    defer free(library);
    options.shared_library_path = library;

    if !generate_android_project(config) {
        log_error("Android loader project generation failed\n");
        return;
    }

    built, apk := android_generate_apk(options, config);

    defer free(apk);

    if !built || !adb_install(apk, reinstall_keeping_data = true) {
        log_error("Android loader APK build or install failed\n");
        return;
    }

    payload := tool_path_join(build_options.output_path, build_options.output_executable_name);

    defer free(payload);
    if !android_copy_file_to_device(payload, config.app_id) log_error("Android loader payload transfer failed\n");
}

#scope_file
#import "Basic";
#import "Compiler";
#load "../../internal/android-toolchain/android-loader.jai";
