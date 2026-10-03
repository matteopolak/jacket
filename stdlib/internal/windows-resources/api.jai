set_icon_by_filename :: (exe_filename: string, icon_filename: string, resource_id: s32 = 2, delete_existing_resources := false) -> bool {
    bytes, read_ok := Files.read_entire_file(icon_filename);
    if !read_ok return false;
    defer B.free(bytes);
    return set_icon_by_data(exe_filename, bytes, resource_id, delete_existing_resources);
}

set_icon_by_data :: (exe_filename: string, icon_data: string, resource_id: s32 = 2, delete_existing_resources := false) -> bool {
    if resource_id < 0 || resource_id > 65535 {
        B.log_error("Icon resource ID is outside the supported numeric range: %", resource_id);
        return false;
    }

    return write_icon_resources(exe_filename, icon_data, cast(u16) resource_id, cast(u16) resource_id, 1033, delete_existing_resources);
}

add_manifest_to_executable :: (exe_filename: string, options: Manifest_Options, resource_id: s32 = 1) -> bool {
    if resource_id < 0 || resource_id > 65535 {
        B.log_error("Manifest resource ID is outside the supported numeric range: %", resource_id);
        return false;
    }

    xml := generate_manifest(options);
    if xml.count == 0 return false;

    defer B.free(xml);

    return write_manifest_resource(exe_filename, xml, cast(u16) resource_id, 1033);
}

disable_runtime_console :: (w: Workspace = -1) {
    options := C.get_build_options(w);
    arguments := ([] string).{ options.additional_linker_arguments.count + 2, cast(*string) B.alloc((options.additional_linker_arguments.count + 2) * size_of(string)) };

    B.assert(arguments.data != null, "linker argument allocation failed");
    // Compiler Build_Options retains its argument slice; keep the backing allocation
    // for the compiler process lifetime instead of invalidating it on return.
    B.array_add(*linker_argument_storage, arguments);
    for argument, index: options.additional_linker_arguments arguments[index] = argument;

    arguments[arguments.count - 2] = "/SUBSYSTEM:windows";
    arguments[arguments.count - 1] = "/ENTRY:mainCRTStartup";
    options.additional_linker_arguments = arguments;
    C.set_build_options(options, w);
}

#scope_file
linker_argument_storage: [..][] string;

Files :: #import "File";
C :: #import "Compiler";
