android_get_external_data_path :: (app_id: string) -> string {
    if !android_device_package_valid(app_id) return "";
    return sprint("/sdcard/Android/data/%/files", app_id);
}

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

android_copy_directories_to_device_quest :: android_copy_directories_to_device;

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
