// Reviewed SDK directory order/cache/server contracts; explicit host-only fixture.
#import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";
S :: #import "String";

warnings: s64;
errors: s64;
assertions: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .WARNING) != 0 warnings += 1;
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    assertions += 1;
    return false;
}

environment :: (name: string, value: string) {
    key := to_c_string(name);
    text := to_c_string(value);

    defer free(key);
    defer free(text);
    result := setenv(key, text, 1);

    assert(result == 0);
}

main :: () {
    context.logger = capture;
    root_name := sprint("%/../../../target/portable-adb", S.path_strip_filename(#file));

    defer free(root_name);
    root, resolved := F.get_absolute_path(root_name);

    assert(resolved);
    defer free(root);
    sdk := sprint("%/SDK space", root);
    alternative := sprint("%/SDK alt", root);
    missing := sprint("%/does not exist", root);
    empty := sprint("%/SDK empty", root);
    failure := sprint("%/start failure SDK", root);

    defer free(sdk);
    defer free(alternative);
    defer free(missing);
    defer free(empty);
    defer free(failure);
    expected := sprint("%/platform-tools/adb", sdk);
    alt_expected := sprint("%/platform-tools/adb", alternative);

    defer free(expected);
    defer free(alt_expected);
    environment("ANDROID_HOME", sdk);
    environment("ANDROID_SDK_ROOT", alternative);
    environment("ANDROID_SDK_HOME", alternative);
    found, path := A.android_find_sdk_path();

    assert(found && path == sdk && warnings == 0);
    free(path);

    storage: Temporary_Storage_Paged_Alloc;
    memory: [65536] u8;

    storage.data = memory.data;
    storage.size = memory.count;
    original_storage := context.temporary_storage;

    context.temporary_storage = *storage.base;
    defer context.temporary_storage = original_storage;
    mark := get_temporary_storage_mark();

    A.__adb_path = "";
    info, valid := A.adb_version();

    assert(valid && info.tool_version == "35.0.2-test" && A.__adb_path == expected);
    set_temporary_storage_mark(mark);
    // Discovery changes, but an established executable cache survives temp reset.
    environment("ANDROID_HOME", alternative);
    found, path = A.android_find_sdk_path();
    assert(found && path == alternative);
    free(path);
    info, valid = A.adb_version();
    assert(valid && A.__adb_path == expected);
    A.__adb_path = "";
    info, valid = A.adb_version();
    assert(valid && A.__adb_path == alt_expected);

    environment("ANDROID_HOME", missing);
    environment("ANDROID_SDK_ROOT", sdk);
    found, path = A.android_find_sdk_path();
    assert(found && path == sdk && warnings == 1);
    free(path);
    environment("ANDROID_SDK_ROOT", missing);
    found, path = A.android_find_sdk_path();
    assert(found && path == alternative && warnings == 3);
    free(path);
    environment("ANDROID_SDK_HOME", missing);
    found, path = A.android_find_sdk_path();
    assert(!found && path == "" && warnings == 6);
    environment("ANDROID_HOME", empty);
    installed, exe := A.is_adb_installed();

    assert(!installed && exe == "");

    environment("ANDROID_HOME", failure);
    A.__adb_path = "";
    original_assertion := context.assertion_failed;

    context.assertion_failed = continue_failure;
    completed, output, diagnostic, code := A.adb("version");

    context.assertion_failed = original_assertion;
    assert(!completed && output == "" && diagnostic == "" && code == 0);
    assert(errors == 1 && assertions == 1);
    reset_temporary_storage(*storage.base);
    print("ok\n");
}

#scope_file
libc :: #system_library "libc";
setenv :: (name: *u8, value: *u8, overwrite: s32) -> s32 #foreign libc;
