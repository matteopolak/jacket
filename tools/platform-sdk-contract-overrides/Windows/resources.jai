// Independent registry discovery; each successful result is caller-owned.
MAKEINTRESOURCE :: (x: s32) -> *u16 {
    return cast(*u16) (cast(u64) (cast,trunc(u16) x));
}

find_windows_kit_root :: (version: string) -> string {
    name, name_ok, name_bytes := Utf8.utf8_to_wide(version);
    if !name_ok return "";
    defer Basic.free(name);
    key: HKEY;
    status := RegOpenKeyExA(HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows Kits\\Installed Roots", 0, KEY_QUERY_VALUE | KEY_WOW64_32KEY, *key);
    if status != 0 return "";
    defer RegCloseKey(key);
    size: DWORD = 0;
    kind: DWORD = 0;
    status = RegQueryValueExW(key, name, null, *kind, null, *size);
    if status != 0 || kind != REG_SZ || size < 2 || size > 0x7ffffffe return "";
    bytes := cast(*u8) Basic.alloc(cast(s64) size + 2);
    if bytes == null return "";
    defer Basic.free(bytes);
    available := size;
    status = RegQueryValueExW(key, name, null, *kind, bytes, *available);
    if status != 0 || kind != REG_SZ || available > size || available % 2 != 0 return "";
    bytes[available] = 0;
    bytes[available + 1] = 0;
    text, converted := Utf8.wide_to_utf8(cast(*u16) bytes);
    if !converted return "";
    return text;
}

find_windows_kit_root :: () -> (root: string, windows_version: s32) {
    current := find_windows_kit_root("KitsRoot10");
    if current.count > 0 return current, 10;
    earlier := find_windows_kit_root("KitsRoot81");
    if earlier.count > 0 return earlier, 8;
    return "", 0;
}

set_icon_by_filename :: (exe_filename: string, icon_filename: string, resource_id: s32 = 2, delete_existing_resources := false) -> bool {
    bytes, read_ok := Files.read_entire_file(icon_filename);
    if !read_ok return false;
    defer Basic.free(bytes.data);
    return set_icon_by_data(exe_filename, bytes, resource_id, delete_existing_resources);
}

set_icon_by_data :: (exe_filename: string, icon_data: string, resource_id: s32 = 2, delete_existing_resources := false) -> bool {
    if exe_filename.count <= 0 || icon_data.count < 6 || icon_data.data == null || resource_id < 1 || resource_id > 65535 return false;
    if read_le16(icon_data.data) != 0 || read_le16(icon_data.data + 2) != 1 return false;
    count := cast(s64) read_le16(icon_data.data + 4);
    if count == 0 || count > 65536 - resource_id return false;
    directory_size := 6 + 16 * count;
    if directory_size > icon_data.count return false;
    // Validate every range before starting a transaction against the executable.
    for index: 0..count-1 {
        entry := icon_data.data + 6 + index * 16;
        size := cast(s64) read_le32(entry + 8);
        offset := cast(s64) read_le32(entry + 12);
        if entry[3] != 0 || size == 0 || offset < directory_size || offset > icon_data.count || size > icon_data.count - offset return false;
    }
    group_size := 6 + 14 * count;
    group := cast(*u8) Basic.alloc(group_size);
    if group == null return false;
    defer Basic.free(group);
    memcpy(group, icon_data.data, 6);
    for index: 0..count-1 {
        source := icon_data.data + 6 + 16 * index;
        target := group + 6 + 14 * index;
        memcpy(target, source, 12);
        write_le16(target + 12, cast(u16) (resource_id + index));
    }
    filename, converted, allocated := Utf8.utf8_to_wide(exe_filename);
    if !converted return false;
    defer Basic.free(filename);
    update := sdk_begin_update(filename, cast(BOOL) delete_existing_resources);
    if update == null return false;
    finished := false;
    defer if !finished sdk_end_update(update, .TRUE);
    for index: 0..count-1 {
        entry := icon_data.data + 6 + 16 * index;
        size := read_le32(entry + 8);
        offset := read_le32(entry + 12);
        if sdk_update(update, MAKEINTRESOURCE(3), MAKEINTRESOURCE(resource_id + cast(s32) index), 0, icon_data.data + offset, size) == .FALSE return false;
    }
    if sdk_update(update, MAKEINTRESOURCE(14), MAKEINTRESOURCE(resource_id), 0, group, cast(DWORD) group_size) == .FALSE return false;
    committed := sdk_end_update(update, .FALSE) != .FALSE;
    finished = true;
    return committed;
}

generate_manifest :: (options: Manifest_Options) -> string {
    builder: Basic.String_Builder;
    defer Basic.free_buffers(*builder);
    Basic.append(*builder, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\">\n<assemblyIdentity type=\"win32\" name=\"Jai.Application\" version=\"1.0.0.0\" processorArchitecture=\"*\"/>\n");
    Basic.append(*builder, "<trustInfo xmlns=\"urn:schemas-microsoft-com:asm.v2\"><security><requestedPrivileges xmlns=\"urn:schemas-microsoft-com:asm.v3\"><requestedExecutionLevel level=\"");
    if options.require_administrator Basic.append(*builder, "requireAdministrator");
    else Basic.append(*builder, "asInvoker");
    Basic.append(*builder, "\" uiAccess=\"false\"/></requestedPrivileges></security></trustInfo>\n");
    if options.dpi_aware || options.use_segment_heap {
        Basic.append(*builder, "<application xmlns=\"urn:schemas-microsoft-com:asm.v3\"><windowsSettings>\n");
        if options.dpi_aware Basic.append(*builder, "<dpiAware xmlns=\"http://schemas.microsoft.com/SMI/2005/WindowsSettings\">true/pm</dpiAware>\n<dpiAwareness xmlns=\"http://schemas.microsoft.com/SMI/2016/WindowsSettings\">PerMonitorV2, PerMonitor</dpiAwareness>\n");
        if options.use_segment_heap Basic.append(*builder, "<heapType xmlns=\"http://schemas.microsoft.com/SMI/2020/WindowsSettings\">SegmentHeap</heapType>\n");
        Basic.append(*builder, "</windowsSettings></application>\n");
    }
    Basic.append(*builder, "</assembly>\n");
    if builder.failed return "";
    return Basic.builder_to_string(*builder);
}

add_manifest_to_executable :: (exe_filename: string, options: Manifest_Options, resource_id: s32 = 1) -> bool {
    if exe_filename.count <= 0 || resource_id < 1 || resource_id > 65535 return false;
    manifest := generate_manifest(options);
    if manifest.count == 0 return false;
    defer Basic.free(manifest.data);
    filename, converted, allocated := Utf8.utf8_to_wide(exe_filename);
    if !converted return false;
    defer Basic.free(filename);
    update := sdk_begin_update(filename, .FALSE);
    if update == null return false;
    if sdk_update(update, MAKEINTRESOURCE(24), MAKEINTRESOURCE(resource_id), 0, manifest.data, cast(DWORD) manifest.count) == .FALSE {
        sdk_end_update(update, .TRUE);
        return false;
    }
    return sdk_end_update(update, .FALSE) != .FALSE;
}

disable_runtime_console :: () {
    sdk_free_console();
}

#scope_file
Basic :: #import "Basic";
Utf8 :: #import "Windows_Utf8";
Files :: #import "File";
sdk_begin_update :: (filename: *u16, delete_existing: BOOL) -> HANDLE #foreign kernel32 "BeginUpdateResourceW";
sdk_update :: (update: HANDLE, resource_type: *u16, resource_name: *u16, language: u16, bytes: *void, count: DWORD) -> BOOL #foreign kernel32 "UpdateResourceW";
sdk_end_update :: (update: HANDLE, discard: BOOL) -> BOOL #foreign kernel32 "EndUpdateResourceW";
sdk_free_console :: () -> BOOL #foreign kernel32 "FreeConsole";
kernel32 :: #system_library "kernel32";
read_le16 :: (bytes: *u8) -> u16 { return cast(u16) bytes[0] | (cast(u16) bytes[1] << 8); }
read_le32 :: (bytes: *u8) -> u32 { return cast(u32) bytes[0] | (cast(u32) bytes[1] << 8) | (cast(u32) bytes[2] << 16) | (cast(u32) bytes[3] << 24); }
write_le16 :: (bytes: *u8, value: u16) { bytes[0] = cast,trunc(u8) value; bytes[1] = cast,trunc(u8) (value >> 8); }
