#import "Basic";
AT :: #import "Android/Toolchain";
F :: #import "File";

main :: () {
    path := AT.android_get_external_data_path("jai.host.fixture");

    defer free(path);
    assert(path == "/sdcard/Android/data/jai.host.fixture/files");
    assert(!AT.android_copy_file_to_device("missing", "jai.host.fixture", "../escape"));
    assert(!AT.android_copy_directories_to_device(string.["missing"], "jai.host.fixture", "/absolute"));
    assert(AT.android_get_external_data_path("../escape") == "");
    assert(AT.android_get_external_data_path("..") == "");
    assert(AT.android_get_external_data_path("jai..fixture") == "");
    assert(AT.android_get_external_data_path("1invalid") == "");
    args := get_command_line_arguments();
    if args.count > 1 {
        AT.__adb_path = args[1];
        AT.adb_device_serial = "host-fixture-serial";
        assert(!AT.adb_install("an apk with spaces.apk", true, true, true,
            uninstall_and_retry_on_key_mismatch = false, log_errors = false));
        assert(!AT.android_copy_file_to_device("payload with spaces.bin", "jai.host.fixture", "files with spaces"));
        config: AT.Android_Project_Config;
        config.app_id = "jai.host.fixture";
        encoded, data := AT.write_binary_app_manifest_to_memory(config);

        assert(encoded);
        defer free(data);
        archive: AT.Zip_Writer;

        AT.zip_init_writer(*archive);
        defer free_buffers(*archive.builder);
        defer array_free(archive.zip_file_infos);
        assert(AT.zip_add_file_from_memory(*archive, data, "AndroidManifest.xml"));
        AT.zip_write_central_directory(*archive);
        bytes := builder_to_string(*archive.builder);

        defer free(bytes);
        assert(F.make_directory_if_it_does_not_exist("generated-device", true));
        assert(F.write_entire_file("generated-device/mismatch.apk", bytes));
        assert(!AT.adb_install("generated-device/mismatch.apk", log_errors = false));
    }

    print("Android device helper host validation passed; no install/device claimed\n");
}
