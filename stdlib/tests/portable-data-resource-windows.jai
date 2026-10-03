// Library-owned native test entry: uses private mechanics without exporting test APIs.
#import "Basic";
F :: #import "File";
Resources :: #import "internal/windows-resources";
Public :: #import "Windows_Resources";
Icons :: #import "internal/resource-wire/icon-directory";
Paths :: #import "internal/files/windows-text";
Win :: #import "Windows";

#assert OS == .WINDOWS;
kernel :: #library,system "kernel32";
FindResourceExW :: (module: Win.HMODULE, resource_type: Win.LPCWSTR, name: Win.LPCWSTR, language: Win.WORD) -> *void #foreign kernel;
SizeofResource :: (module: Win.HMODULE, resource: *void) -> Win.DWORD #foreign kernel;
LoadResource :: (module: Win.HMODULE, resource: *void) -> Win.HGLOBAL #foreign kernel;
LockResource :: (resource: Win.HGLOBAL) -> *void #foreign kernel;

view_resource :: (module: Win.HMODULE, kind: u16, id: u16) -> string {
    resource := FindResourceExW(module, cast(*u16) cast(u64) kind, cast(*u16) cast(u64) id, 1033);

    assert(resource != null);
    size := SizeofResource(module, resource);

    assert(size > 0);
    loaded := LoadResource(module, resource);

    assert(loaded != null);
    data := LockResource(loaded);

    assert(data != null);

    return .{ size, cast(*u8) data };
}

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);
    source := sprint("%/resource-only.dll", args[1]);
    icon_path := sprint("%/resource.ico", args[1]);
    destination := sprint("%/updated-字.dll", args[1]);

    defer free(source);
    defer free(icon_path);
    defer free(destination);
    original, original_ok := F.read_entire_file(source);
    icon, icon_ok := F.read_entire_file(icon_path);

    assert(original_ok && icon_ok);
    defer free(original);
    defer free(icon);
    assert(F.write_entire_file(destination, original));
    assert(Resources.write_icon_resources(destination, icon, 501, 600, 1033, false));
    manifest := "<?xml version=\"1.0\" encoding=\"UTF-8\"?><assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\"><assemblyIdentity type=\"win32\" name=\"portable.data.fixture\" version=\"1.0.0.0\"/></assembly>";

    assert(Resources.write_manifest_resource(destination, manifest, 1, 1033));
    valid, directory := Icons.inspect(icon);

    assert(valid);
    defer Icons.release(*directory);
    group_ok, group := Icons.make_group(directory, 600);

    assert(group_ok);
    defer free(group);
    path := Paths.to_utf16(destination);

    assert(path != null);
    defer free(path);
    module := Win.LoadLibraryExW(path, null, 2);

    assert(module != null);
    assert(view_resource(module, 14, 501) == group);
    assert(view_resource(module, 3, 600) == directory.images[0].bytes);
    assert(view_resource(module, 24, 1) == manifest);
    assert(view_resource(module, 14, 101).count == 76);
    assert(Win.FreeLibrary(module) != .FALSE);
    assert(Resources.write_icon_resources(destination, icon, 700, 800, 1033, true));
    module = Win.LoadLibraryExW(path, null, 2);
    assert(module != null);
    assert(FindResourceExW(module, cast(*u16) 14, cast(*u16) 101, 1033) == null);
    assert(FindResourceExW(module, cast(*u16) 14, cast(*u16) 501, 1033) == null);
    assert(FindResourceExW(module, cast(*u16) 24, cast(*u16) 1, 1033) == null);
    assert(view_resource(module, 14, 700).count == 76);
    assert(view_resource(module, 3, 800) == directory.images[0].bytes);
    assert(Win.FreeLibrary(module) != .FALSE);
    assert(Public.set_icon_by_filename(destination, icon_path));
    assert(Public.set_icon_by_data(destination, icon, resource_id = 900));
    default_options: Public.Manifest_Options;

    assert(Public.add_manifest_to_executable(destination, default_options));
    expected_manifest := Public.generate_manifest(default_options);

    defer free(expected_manifest);
    module = Win.LoadLibraryExW(path, null, 2);
    assert(module != null);
    assert(view_resource(module, 14, 2).count == 76);
    assert(view_resource(module, 14, 900).count == 76);
    assert(view_resource(module, 24, 1) == expected_manifest);
    assert(Win.FreeLibrary(module) != .FALSE);
    print("native Windows resource transaction and SDK readback ok\n");
}
