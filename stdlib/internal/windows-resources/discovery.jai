find_windows_kit_root :: (version: string) -> string {
    return registry_text("SOFTWARE\\Microsoft\\Windows Kits\\Installed Roots", version);
}

find_windows_kit_root :: () -> (root: string, windows_version: s32) {
    ten := find_windows_kit_root("KitsRoot10");
    selected := select_sdk_directory(ten, 4, "");

    if selected.count > 0 return selected, 10;

    eight := find_windows_kit_root("KitsRoot81");

    selected = select_sdk_directory(eight, 2, "winv");
    if selected.count > 0 return selected, 8;

    return "", 0;
}

find_visual_studio_in_a_ridiculous_garbage_way :: () -> (library_path := "", linker_path := "") {
    library, linker := discover_setup_instances();
    if library.count > 0 return library, linker;

    versions: [4] string = .["14.0", "12.0", "11.0", "10.0"];
    for version: versions {
        installation := registry_text("SOFTWARE\\Microsoft\\VisualStudio\\SxS\\VS7", version);
        if installation.count == 0 continue;

        library = B.tprint("%/VC/Lib/amd64", installation);
        linker = B.tprint("%/VC/bin", installation);
        if has_runtime_library(library) return library, linker;
    }

    return "", "";
}

#scope_file
R :: #import "Windows_Registry";
Files :: #import "File";
Utilities :: #import "File_Utilities";
Strings :: #import "String";
Versions :: #import "internal/resource-wire/tool-versions";

registry_text :: (key_name: string, value_name: string) -> string {
    key_path := Paths.to_utf16(key_name);
    value_path := Paths.to_utf16(value_name);

    defer B.free(key_path);
    defer B.free(value_path);
    if !key_path || !value_path return "";

    views: [2] u32 = .[R.KEY_WOW64_32KEY, R.KEY_WOW64_64KEY];
    for view: views {
        key: W.HKEY;
        // SDK HKEY_LOCAL_MACHINE is a sign-extended LONG pseudo handle.
        machine := cast(W.HKEY) cast(s64) -2147483646;

        if R.RegOpenKeyExW(machine, key_path, 0, R.KEY_READ | view, *key) != 0 continue;

        defer close_registry_key(key);
        kind: u32;
        byte_count: u32;

        if R.RegQueryValueExW(key, value_path, null, *kind, null, *byte_count) != 0 || kind != 1 || byte_count < 2 || byte_count % 2 != 0 continue;

        storage := cast(*u8) B.alloc(cast(s64) byte_count + 2);
        if !storage return "";

        defer B.free(storage);
        capacity := byte_count;
        if R.RegQueryValueExW(key, value_path, null, *kind, storage, *capacity) != 0 || kind != 1 || capacity > byte_count || capacity % 2 != 0 continue;

        units := cast(*u16) storage;
        units[capacity / 2] = 0;
        count := cast(s32) (capacity / 2);
        if count > 0 && units[count - 1] == 0 count -= 1;

        converted, valid := Paths.from_utf16(units, count);
        if !valid continue;

        defer B.free(converted);

        return B.copy_temporary_string(converted);
    }

    return "";
}

close_registry_key :: (key: W.HKEY) {
    status := R.RegCloseKey(key);
    if status != 0 B.log_error("Registry key close failed (Win32 error %)", status);
} @Releases

Version_Selection :: struct {
    fields: s32;
    prefix: string;
    best: string;
    digits: [4] u32;
}

consider_directory :: (info: *Utilities.File_Visit_Info, selection: *Version_Selection) {
    if info.had_error || !info.is_directory return;

    valid, digits := Versions.parse_version(info.short_name, selection.fields, selection.prefix);
    if !valid || (selection.best.count > 0 && !Versions.is_later(digits, selection.digits)) return;

    B.free(selection.best);
    selection.best = B.copy_string(info.full_name);
    selection.digits = digits;
}

select_sdk_directory :: (root: string, fields: s32, prefix: string) -> string {
    if root.count == 0 return "";

    directory := B.sprint("%/Lib", root);

    defer B.free(directory);
    selection := Version_Selection.{ fields = fields, prefix = prefix };
    success := Utilities.visit_files(directory, false, *selection, consider_directory, visit_files = false, visit_directories = true, visit_symlinks = false, follow_directory_symlinks = false);

    if !success {
        B.free(selection.best);
        return "";
    }

    return selection.best;
}

#scope_module
has_runtime_library :: (library: string) -> bool {
    filename := B.tprint("%/vcruntime.lib", library);
    return Utilities.file_exists(filename);
}
