android_find_sdk_path :: () -> bool, string {
    S :: #import "System";
    home, have_home := S.get_home_directory();
    conventional: string;

    if have_home {
        #if OS == .WINDOWS conventional = tool_path_join(home, "AppData/Local/Android/Sdk");
        else conventional = tool_path_join(home, "Android/Sdk");
    }

    defer free(conventional);
    primary := tool_environment("ANDROID_HOME");
    legacy_root := tool_environment("ANDROID_SDK_ROOT");
    legacy_home := tool_environment("ANDROID_SDK_HOME");

    defer free(primary);
    defer free(legacy_root);
    defer free(legacy_home);

    return select_sdk_directory(string.[conventional, primary, legacy_root, legacy_home]);
}

is_adb_installed :: () -> bool, string {
    F :: #import "File_Utilities";
    found, sdk := android_find_sdk_path();

    defer free(sdk);
    if !found return false, "";

    #if OS == .WINDOWS path := tool_path_join(sdk, "platform-tools/adb.exe");
    else path := tool_path_join(sdk, "platform-tools/adb");
    defer free(path);
    if !F.file_exists(path) return false, "";

    return true, copy_temporary_string(path);
}

#scope_module
#import "Basic";

select_sdk_directory :: (paths: [] string) -> bool, string {
    F :: #import "File_Utilities";

    for path, index: paths {
        if path.count == 0 continue;
        directory, valid := F.is_directory(path);
        if directory && valid return true, copy_string(path);
        if index > 0 context.logger("Android SDK environment path is not a directory\n", context.logger_data, Log_Info.{ common_flags = .WARNING });
    }

    return false, "";
}

tool_path_join :: (root: string, suffix: string) -> string {
    while root.count > 1 && (root[root.count - 1] == 47 || root[root.count - 1] == 92) root.count -= 1;

    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, root);
    if root.count > 0 && root[root.count - 1] != 47 && root[root.count - 1] != 92 append(*builder, "/");

    append(*builder, suffix);

    return builder_to_string(*builder);
}
