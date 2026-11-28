// Returns the app-specific external storage folder of `app_id` on the device.
//
// The result is `/sdcard/Android/data/<app_id>/files`, allocated with `sprint`; the caller frees it. Returns
// an empty string when `app_id` is not a dotted package name of letters, digits and underscores (no component
// may start with a digit).
android_get_external_data_path :: (app_id: string) -> string {
    if !android_device_package_valid(app_id) return "";
    return sprint("/sdcard/Android/data/%/files", app_id);
}

// Copies the local file `file_path` into the app's external storage folder on the device.
//
// `external_storage_relative_path` is a folder below that root; it may not be absolute, contain `..`
// components or contain `:` or NUL. The folder is created on the device first. Returns `false` when an
// argument is rejected or `adb` fails.
android_copy_file_to_device :: (file_path: string, app_id: string, external_storage_relative_path := "") -> bool {
    if !android_device_relative_valid(external_storage_relative_path) return false;

    root := android_get_external_data_path(app_id);

    defer free(root);
    if root.count == 0 return false;

    destination := tool_path_join(root, external_storage_relative_path);

    defer free(destination);
    if !adb_shell_mkdir(destination) return false;

    return adb_push(file_path, destination);
}

// Copies each folder in `directories` into the app's external storage folder on the device.
//
// `dest_relative_path` follows the rules of [android_copy_file_to_device]. With `only_update_newer_files` the
// push uses `--sync`; with `force_create_directories` the destination is created first. Returns `false` at the
// first source that is not a directory or whose push fails; earlier folders stay copied.
android_copy_directories_to_device :: (directories: [] string, app_id: string, dest_relative_path := "", only_update_newer_files := false, force_create_directories := false) -> bool {
    if !android_device_relative_valid(dest_relative_path) return false;

    root := android_get_external_data_path(app_id);

    defer free(root);
    if root.count == 0 return false;

    destination := tool_path_join(root, dest_relative_path);

    defer free(destination);
    if force_create_directories && !adb_shell_mkdir(destination) return false;

    for directory: directories {
        valid_directory, valid := Android_Device_Files.is_directory(directory);
        if !valid_directory || !valid return false;
        if !adb_push(directory, destination, only_update_newer_files) return false;
    }

    return true;
}

// Is another name for [android_copy_directories_to_device].
android_copy_directories_to_device_quest :: android_copy_directories_to_device;

// Picks the device that later `adb` calls target, at compile time.
//
// Logs an error and returns without selecting when `adb` is not installed, no ready device is attached, or
// several are ready and none matches. A device whose serial equals `adb_device_serial` wins; otherwise one
// whose model is `preferred_model_name` (the last match); otherwise the only ready device. The choice is
// stored in `adb_device_serial`. `required_reason` completes the error messages.
android_check_for_adb_and_select_device :: (required_reason := "by metaprogram", preferred_model_name := "") #compile_time {
    installed := is_adb_installed();
    if !installed {
        log_error("ADB is required % but was not found\n", required_reason);
        return;
    }

    success, devices := adb_devices(true);
    if !success || devices.count == 0 {
        log_error("No ready Android device is available %\n", required_reason);
        return;
    }

    selected := -1;

    for device, index: devices {
        if adb_device_serial.count > 0 && device.serial == adb_device_serial {
            selected = index;
            break;
        }

        if preferred_model_name.count > 0 && device.model_name == preferred_model_name selected = index;
    }

    if selected < 0 && devices.count == 1 selected = 0;

    if selected < 0 {
        log_error("Multiple Android devices are ready; set a serial or preferred model\n");
        return;
    }

    android_device_retain_serial(devices[selected].serial);
}

// Installs the APK at `apk_path` with `adb install`; returns `true` on success.
//
// - reinstall_keeping_data: passes `-r`.
// - allow_test_apk: passes `-t`.
// - allow_version_code_downgrade: passes `-d`.
// - uninstall_and_retry_on_key_mismatch: when the install fails with `INSTALL_FAILED_UPDATE_INCOMPATIBLE`,
//   reads the package name from the APK's manifest, uninstalls that package (deleting its data) and installs
//   again.
// - log_errors: logs a failed `adb` command.
adb_install :: (apk_path: string, reinstall_keeping_data := false, allow_test_apk := false, allow_version_code_downgrade := false, uninstall_and_retry_on_key_mismatch := true, log_errors := true) -> bool {
    arguments: [..] string;

    defer array_free(arguments);
    array_add(*arguments, "install");
    if reinstall_keeping_data array_add(*arguments, "-r");
    if allow_test_apk array_add(*arguments, "-t");
    if allow_version_code_downgrade array_add(*arguments, "-d");

    array_add(*arguments, apk_path);
    success, output, errors := adb(..arguments, log_adb_command_errors = log_errors);

    defer free(output);
    defer free(errors);
    if success return true;
    if !uninstall_and_retry_on_key_mismatch || (!Android_Device_Strings.contains(output, "INSTALL_FAILED_UPDATE_INCOMPATIBLE") && !Android_Device_Strings.contains(errors, "INSTALL_FAILED_UPDATE_INCOMPATIBLE")) return false;

    read_ok, manifest := read_binary_manifest_from_apk(apk_path);
    if !read_ok return false;

    defer deinit(*manifest);
    package := android_device_manifest_package(manifest);
    if package.count == 0 || !adb_uninstall(package) return false;

    retried, retry_output, retry_errors := adb(..arguments, log_adb_command_errors = log_errors);

    defer free(retry_output);
    defer free(retry_errors);

    return retried;
}

#scope_file
#import "Basic";
Android_Device_Files :: #import "File_Utilities";
Android_Device_Strings :: #import "String";
#load "../../internal/android-toolchain/android-device.jai";
