visit_files :: (asset_manager: *AAssetManager, dir_name: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), visit_files := true, visit_directories := false, visit_symlinks := false, follow_directory_symlinks := false) -> bool {
    if asset_manager == null || proc == null || !_sdk_asset_path(dir_name) return false;
    if !visit_files && !visit_directories return true;
    // APK assets have no symlink entries. The NDK directory iterator supplies
    // files only; Java AssetManager.list is needed to discover child directories.
    if !recursive && !visit_directories {
        path := to_c_string(dir_name);
        if path == null return false;
        defer free(path);
        directory := AAssetManager_openDir(asset_manager, path);
        if directory == null return false;
        defer AAssetDir_close(directory);
        while true {
            native_name := AAssetDir_getNextFileName(directory);
            if native_name == null break;
            name := _sdk_asset_c_string(native_name);
            full := _sdk_asset_join(dir_name, name);
            defer free(full);
            info := File_Visit_Info.{name, full, false};
            proc(*info, user_data);
        }
        return true;
    }
    app := context.android_app;
    if app == null || app.activity == null || app.activity.vm == null return false;
    if app.activity.assetManager != asset_manager return false;
    vm := app.activity.vm;
    env: *JNIEnv;
    attached := false;
    state := vm.*.GetEnv(vm, cast(**void) *env, JNI_VERSION_1_6);
    if state == JNI_EDETACHED {
        if vm.*.AttachCurrentThread(vm, *env, null) != JNI_OK return false;
        attached = true;
    } else if state != JNI_OK return false;
    defer if attached vm.*.DetachCurrentThread(vm);
    if env == null || env.*.ExceptionCheck(env) != 0 return false;
    activity_class := env.*.GetObjectClass(env, app.activity.clazz);
    if _sdk_asset_jni_failed(env) || activity_class == null return false;
    defer env.*.DeleteLocalRef(env, activity_class);
    get_assets := env.*.GetMethodID(env, activity_class, cast(*u8) "getAssets\0", cast(*u8) "()Landroid/content/res/AssetManager;\0");
    if _sdk_asset_jni_failed(env) || get_assets == null return false;
    java_assets := env.*.CallObjectMethodA(env, app.activity.clazz, get_assets, null);
    if _sdk_asset_jni_failed(env) || java_assets == null return false;
    defer env.*.DeleteLocalRef(env, java_assets);
    if _sdk_asset_manager_from_java(env, java_assets) != asset_manager return false;
    asset_class := env.*.GetObjectClass(env, java_assets);
    if _sdk_asset_jni_failed(env) || asset_class == null return false;
    defer env.*.DeleteLocalRef(env, asset_class);
    list := env.*.GetMethodID(env, asset_class, cast(*u8) "list\0", cast(*u8) "(Ljava/lang/String;)[Ljava/lang/String;\0");
    if _sdk_asset_jni_failed(env) || list == null return false;
    return _sdk_asset_walk(asset_manager, env, java_assets, list, dir_name, recursive, user_data, proc, visit_files, visit_directories, 0);
}
// PRIVATE HELPERS
#import "Android/Jni";
#import "File";
Asset_Unicode :: #import "Unicode";
_sdk_asset_android :: #system_library "libandroid";
_sdk_asset_manager_from_java :: (env: *JNIEnv, manager: jobject) -> *AAssetManager #foreign _sdk_asset_android "AAssetManager_fromJava";
_sdk_asset_jni_failed :: (env: *JNIEnv) -> bool #no_context {
    if env.*.ExceptionCheck(env) == 0 return false;
    // Translate exceptions raised by our own JNI calls to the bool result.
    env.*.ExceptionClear(env);
    return true;
}
_sdk_asset_c_string :: (pointer: *u8) -> string #no_context {
    count: s64 = 0;
    while pointer[count] != 0 count += 1;
    return string.{count, pointer};
}
_sdk_asset_path :: (path: string) -> bool #no_context {
    if path.count < 0 || (path.count > 0 && path.data == null) return false;
    if path.count > 0 && (path[0] == 47 || path[0] == 92) return false;
    start: s64 = 0;
    for byte, index: path {
        if byte == 0 || byte == 92 return false;
        if byte == 47 {
            if index == start || (index - start == 2 && path[start] == 46 && path[start + 1] == 46) return false;
            start = index + 1;
        }
    }
    if path.count - start == 2 && path[start] == 46 && path[start + 1] == 46 return false;
    return true;
}
_sdk_asset_join :: (directory: string, name: string) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);
    append(*builder, directory);
    if directory.count > 0 && directory[directory.count - 1] != 47 append(*builder, cast(u8) 47);
    append(*builder, name);
    return builder_to_string(*builder);
}
_sdk_asset_java_string :: (env: *JNIEnv, text: string) -> jstring {
    if text.count < 0 || text.count > 0x3fffffff return null;
    units := cast(*u16) alloc((text.count + 1) * 2);
    if units == null return null;
    defer free(units);
    input: s64 = 0; output: s32 = 0;
    while input < text.count {
        scalar, consumed, status := Asset_Unicode.character_utf8_to_utf32(text.data + input, text.count - input);
        if status != .CONVERSION_OK return null;
        input += consumed;
        if scalar <= 0xffff { units[output] = cast(u16) scalar; output += 1; }
        else {
            scalar -= 0x10000;
            units[output] = cast(u16) (0xd800 + (scalar >> 10));
            units[output + 1] = cast(u16) (0xdc00 + (scalar & 0x3ff));
            output += 2;
        }
    }
    return env.*.NewString(env, units, output);
}
_sdk_asset_from_java :: (env: *JNIEnv, value: jstring) -> string, bool {
    count := env.*.GetStringLength(env, value);
    if _sdk_asset_jni_failed(env) || count < 0 return "", false;
    characters := env.*.GetStringChars(env, value, null);
    if _sdk_asset_jni_failed(env) || characters == null return "", false;
    defer env.*.ReleaseStringChars(env, value, characters);
    builder: String_Builder;
    defer free_buffers(*builder);
    index: s32 = 0;
    while index < count {
        scalar := cast(u32) characters[index]; index += 1;
        if scalar >= 0xd800 && scalar <= 0xdbff {
            if index == count return "", false;
            low := cast(u32) characters[index]; index += 1;
            if low < 0xdc00 || low > 0xdfff return "", false;
            scalar = 0x10000 + ((scalar - 0xd800) << 10) + low - 0xdc00;
        } else if scalar >= 0xdc00 && scalar <= 0xdfff return "", false;
        buffer: [4] u8;
        encoded := string.{4, *buffer[0]};
        if Asset_Unicode.character_utf32_to_utf8(scalar, *encoded) != .CONVERSION_OK return "", false;
        append(*builder, encoded);
    }
    return builder_to_string(*builder), true;
}
_sdk_asset_walk :: (manager: *AAssetManager, env: *JNIEnv, assets: jobject, list: jmethodID, directory: string, recursive: bool, user_data: $T, proc: (info: *File_Visit_Info, user_data: T), files: bool, directories: bool, depth: s64) -> bool {
    if depth >= 256 return false;
    java_path := _sdk_asset_java_string(env, directory);
    if _sdk_asset_jni_failed(env) || java_path == null return false;
    defer env.*.DeleteLocalRef(env, java_path);
    arguments: [1] jvalue; arguments[0].l = java_path;
    names := env.*.CallObjectMethodA(env, assets, list, *arguments[0]);
    if _sdk_asset_jni_failed(env) || names == null return false;
    defer env.*.DeleteLocalRef(env, names);
    count := env.*.GetArrayLength(env, names);
    if _sdk_asset_jni_failed(env) || count < 0 return false;
    index: s32 = 0;
    while index < count {
        java_name := env.*.GetObjectArrayElement(env, names, index); index += 1;
        if _sdk_asset_jni_failed(env) || java_name == null return false;
        defer env.*.DeleteLocalRef(env, java_name);
        name, converted := _sdk_asset_from_java(env, java_name);
        if !converted return false;
        if name.count == 0 return false;
        defer free(name);
        if !_sdk_asset_path(name) return false;
        for byte: name if byte == 47 return false;
        full := _sdk_asset_join(directory, name);
        defer free(full);
        native_path := to_c_string(full);
        if native_path == null return false;
        defer free(native_path);
        asset := AAssetManager_open(manager, native_path, cast(AASSET_MODE) 0);
        if asset != null {
            AAsset_close(asset);
            if files {
                info := File_Visit_Info.{name, full, false};
                proc(*info, user_data);
            }
        } else {
            info := File_Visit_Info.{name, full, recursive};
            if directories proc(*info, user_data);
            if recursive && info.descend_into_directory && !_sdk_asset_walk(manager, env, assets, list, full, true, user_data, proc, files, directories, depth + 1) return false;
        }
    }
    return true;
}
