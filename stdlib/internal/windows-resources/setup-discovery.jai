discover_setup_instances :: () -> string, string {
    initialized := CoInitializeEx(null, 0);
    if initialized < 0 && cast,no_check(u32) initialized != 0x80010106 return "", "";

    defer if initialized >= 0 CoUninitialize();
    class_id := W.GUID.{ 0x177f0c4a, 0x1cd3, 0x4de7, .[0xa3, 0x2c, 0x71, 0xdb, 0xbb, 0x9f, 0xa3, 0x6d] };
    interface_id := W.GUID.{ 0x42843719, 0xdb4c, 0x46c2, .[0x8e, 0x7c, 0x64, 0xf1, 0x81, 0x6e, 0xfd, 0x5b] };
    configuration: *Setup;

    if CoCreateInstance(*class_id, null, 1, *interface_id, cast(**void) *configuration) < 0 || !configuration return "", "";

    defer configuration.vtable.unknown.Release(cast(*W.IUnknown) configuration);
    enumeration: *Enumeration;
    if configuration.vtable.enumerate(configuration, *enumeration) < 0 || !enumeration return "", "";

    defer enumeration.vtable.unknown.Release(cast(*W.IUnknown) enumeration);

    while true {
        instance: *Instance;
        fetched: u32;
        status := enumeration.vtable.next(enumeration, 1, *instance, *fetched);

        if status < 0 || fetched == 0 break;
        if !instance break;

        defer instance.vtable.unknown.Release(cast(*W.IUnknown) instance);
        native_path: *u16;
        path_status := instance.vtable.path(instance, *native_path);

        defer if native_path SysFreeString(native_path);
        if path_status < 0 || !native_path continue;

        length := SysStringLen(native_path);
        if length > 0x7fffffff continue;

        installation, valid := Paths.from_utf16(native_path, cast(s32) length);
        if !valid continue;

        defer B.free(installation);
        version_filename := B.sprint("%/VC/Auxiliary/Build/Microsoft.VCToolsVersion.default.txt", installation);

        defer B.free(version_filename);
        text, read_ok := Files.read_entire_file(version_filename, log_errors = false);
        if !read_ok continue;

        defer B.free(text);
        version := Strings.trim(text);
        if version.count == 0 continue;

        library := B.tprint("%/VC/Tools/MSVC/%/lib/x64", installation, version);
        if !has_runtime_library(library) continue;

        linker := B.tprint("%/VC/Tools/MSVC/%/bin/Hostx64/x64", installation, version);

        return library, linker;
    }

    return "", "";
}

#scope_file
Files :: #import "File";
Strings :: #import "String";

// ABI prefixes from Microsoft's Setup.Configuration.h, Native SDK 3.14.2075.
Setup :: struct {
    vtable: *Setup_Table;
}

Enumeration :: struct {
    vtable: *Enumeration_Table;
}

Instance :: struct {
    vtable: *Instance_Table;
}

Setup_Table :: struct {
    unknown: W.IUnknown_VTable;
    enumerate: (this: *Setup, result: **Enumeration) -> s32 #cpp_method;
}

Enumeration_Table :: struct {
    unknown: W.IUnknown_VTable;
    next: (this: *Enumeration, count: u32, result: **Instance, fetched: *u32) -> s32 #cpp_method;
}

Instance_Table :: struct {
    unknown: W.IUnknown_VTable;
    instance_id_slot: *void;
    install_date_slot: *void;
    installation_name_slot: *void;
    path: (this: *Instance, result: **u16) -> s32 #cpp_method;
}

ole :: #library,system "ole32";
automation :: #library,system "oleaut32";
CoInitializeEx :: (reserved: *void, mode: u32) -> s32 #foreign ole;
CoUninitialize :: () #foreign ole;
CoCreateInstance :: (class_id: *W.GUID, outer: *W.IUnknown, context: u32, interface_id: *W.GUID, result: **void) -> s32 #foreign ole;
SysFreeString :: (text: *u16) #foreign automation;
SysStringLen :: (text: *u16) -> u32 #foreign automation;
