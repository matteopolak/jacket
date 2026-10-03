android_build_or_update_dynamic_loader :: (project_config: Android_Project_Config, build_options: Build_Options, android_build_options: Android_Build_Options, force_rebuild_loader := false) -> bool {
    return android_dynamic_loader_build(project_config, build_options, android_build_options, force_rebuild_loader);
}

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
