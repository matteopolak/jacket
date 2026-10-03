find_visual_studio_in_a_ridiculous_garbage_way :: () -> (library_path := "", linker_path := "") {
    // S_OK and S_FALSE both require balancing CoUninitialize. A changed apartment
    // already supplies COM; this function must not uninitialize someone else's.
    initialized := CoInitializeEx(null, 0);
    if initialized < 0 && initialized != -2147417850 return "", "";
    defer if initialized >= 0 CoUninitialize();
    clsid := uid("177F0C4A-1CD3-4DE7-A32C-71DBBB9FA36D");
    iid := uid(ISetupConfiguration.uuid);
    configuration: *ISetupConfiguration;
    if CoCreateInstance(*clsid, null, cast(CLSCTX) 1, *iid, cast(**void) *configuration) < 0 || configuration == null return "", "";
    defer IUnknown_Release(cast(*IUnknown) configuration);
    instances: *IEnumSetupInstances;
    if configuration.vtable.EnumInstances(configuration, *instances) < 0 || instances == null return "", "";
    defer IUnknown_Release(cast(*IUnknown) instances);
    best_version: [4] u32;
    while true {
        instance: *ISetupInstance;
        fetched: u32;
        result := instances.vtable.Next(instances, 1, *instance, *fetched);
        if result < 0 || fetched == 0 break;
        if instance == null break;
        defer IUnknown_Release(cast(*IUnknown) instance);
        path_bstr: BSTR; version_bstr: BSTR;
        if instance.vtable.GetInstallationPath(instance, *path_bstr) < 0 || path_bstr == null continue;
        defer SysFreeString(path_bstr);
        if instance.vtable.GetInstallationVersion(instance, *version_bstr) < 0 || version_bstr == null continue;
        defer SysFreeString(version_bstr);
        installation, path_ok := wide_to_utf8(path_bstr);
        if !path_ok continue;
        defer free(installation);
        version, version_ok := wide_to_utf8(version_bstr);
        if !version_ok continue;
        defer free(version);
        components, parsed := _sdk_vs_version(version);
        if !parsed continue;
        if library_path.count > 0 && !_sdk_vs_newer(components, best_version) continue;
        version_file := _sdk_vs_join(installation, "VC\\Auxiliary\\Build\\Microsoft.VCToolsVersion.default.txt");
        defer free(version_file);
        tools_version, read_ok := read_entire_file(version_file);
        if !read_ok || tools_version.count == 0 continue;
        defer free(tools_version);
        start: s64 = 0; end := tools_version.count;
        while start < end && tools_version[start] <= 32 start += 1;
        while end > start && tools_version[end - 1] <= 32 end -= 1;
        trimmed := string.{end - start, tools_version.data + start};
        tool_components, tools_parsed := _sdk_vs_version(trimmed);
        if !tools_parsed continue;
        tools_prefix := _sdk_vs_join(installation, "VC\\Tools\\MSVC");
        defer free(tools_prefix);
        tools_root := _sdk_vs_join(tools_prefix, trimmed);
        defer free(tools_root);
        candidate_library: string;
        candidate_linker: string;
        #if CPU == .ARM64 {
            candidate_library = _sdk_vs_join(tools_root, "lib\\arm64");
            candidate_linker = _sdk_vs_join(tools_root, "bin\\Hostarm64\\arm64\\link.exe");
            if !_sdk_vs_file(candidate_linker) {
                free(candidate_linker);
                candidate_linker = _sdk_vs_join(tools_root, "bin\\Hostx64\\arm64\\link.exe");
            }
        } else #if CPU == .X64 {
            candidate_library = _sdk_vs_join(tools_root, "lib\\x64");
            candidate_linker = _sdk_vs_join(tools_root, "bin\\Hostx64\\x64\\link.exe");
        } else {
            // The supplied declarations target Win64 only.
            continue;
        }
        runtime_lib := _sdk_vs_join(candidate_library, "vcruntime.lib");
        defer free(runtime_lib);
        valid := _sdk_vs_file(runtime_lib) && _sdk_vs_file(candidate_linker);
        if !valid { free(candidate_library); free(candidate_linker); continue; }
        if library_path.count > 0 free(library_path);
        if linker_path.count > 0 free(linker_path);
        library_path = candidate_library;
        linker_path = candidate_linker;
        best_version = components;
    }
    return library_path, linker_path;
}
// PRIVATE HELPERS
_sdk_vs_join :: (left: string, right: string) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);
    append(*builder, left);
    if left.count > 0 && left[left.count - 1] != 92 && left[left.count - 1] != 47 append(*builder, cast(u8) 92);
    append(*builder, right);
    return builder_to_string(*builder);
}
_sdk_vs_file :: (path: string) -> bool {
    wide, success, bytes := utf8_to_wide(path);
    if !success || wide == null return false;
    defer free(wide);
    attributes := GetFileAttributesW(wide);
    return attributes != 0xffffffff && (attributes & 16) == 0;
}
_sdk_vs_version :: (text: string) -> [4] u32, bool {
    values: [4] u32;
    if text.count == 0 return values, false;
    component: s64 = 0;
    digits := false;
    for character: text {
        if character == 46 {
            if !digits || component == 3 return values, false;
            component += 1; digits = false;
        } else {
            if character < 48 || character > 57 return values, false;
            digit := cast(u32) character - 48;
            if values[component] > (0xffffffff - digit) / 10 return values, false;
            values[component] = values[component] * 10 + digit;
            digits = true;
        }
    }
    return values, digits;
}
_sdk_vs_newer :: (left: [4] u32, right: [4] u32) -> bool {
    for index: 0..3 {
        if left[index] > right[index] return true;
        if left[index] < right[index] return false;
    }
    return false;
}
