#import "Basic";
B :: #import "Bindings_Generator";
P :: #import "POSIX";

dispose :: (library: *B.Library_Info) {
    if library.handle assert(P.dlclose(library.handle) == 0);
    for name: library.symbol_names free(name);

    array_free(library.symbol_names);
    free(library.identifier);
    free(library.full_path);
    free(library);
}

main :: () {
    paths := string.["./images"];
    shared := B.os_load_library(paths, "Resolved", "liboriginal_resolved", false);

    assert(shared != null && shared.is_shared_library && !shared.is_system_library);
    defer dispose(shared);
    assert(shared.full_path == "./images/liboriginal_resolved");
    assert(B.os_symbol_exists_in_library(shared, "original_resolved"));
    function: #type () -> s32 #c_call;
    name := to_c_string("original_resolved");

    defer free(name);
    function = cast(type_of(function)) P.dlsym(shared.handle, name);
    assert(function != null && function() == 37);
    assert(!B.os_symbol_exists_in_library(shared, "original_static_only"));
    other := B.os_load_library(string.["./other"], "Other", "liboriginal_resolved", true);

    assert(other != null && other.full_path == "liboriginal_resolved");
    defer dispose(other);
    function = cast(type_of(function)) P.dlsym(other.handle, name);
    assert(function != null && function() == 73);

    system := B.os_load_library(paths, "", "liboriginal_resolved.dylib", true);

    assert(system != null && system.full_path == "liboriginal_resolved");
    defer dispose(system);
    assert(system.identifier == "liboriginal_resolved");
    assert(system.is_system_library && system.is_shared_library);
    assert(B.os_symbol_exists_in_library(system, "original_resolved"));

    archive := B.os_load_library(paths, "Archive", "original_archive.a", false);

    assert(archive != null && !archive.is_shared_library && !archive.handle);
    defer dispose(archive);
    assert(archive.full_path == "./images/original_archive");
    assert(B.os_symbol_exists_in_library(archive, "original_resolved"));
    assert(B.os_symbol_exists_in_library(archive, "original_static_only"));
    assert(!B.os_symbol_exists_in_library(archive, "missing_original"));

    for filename: string.["libOriginal-Resolved.native", "liboriginal_version.1.2.dylib", "liboriginal_elf.so.3.4"] {
        suffix := ifx filename == "libOriginal-Resolved.native" then ".native" else ifx filename == "liboriginal_elf.so.3.4" then ".so" else ".dylib";
        variant := B.os_load_library(paths, "", filename, false, shared_library_suffix = suffix);

        assert(variant != null && variant.is_shared_library);
        expected := ifx filename == "libOriginal-Resolved.native" then "./images/libOriginal-Resolved" else ifx filename == "liboriginal_elf.so.3.4" then "./images/liboriginal_elf" else "./images/liboriginal_version";

        assert(variant.full_path == expected);
        assert(B.os_symbol_exists_in_library(variant, "original_resolved"));
        if filename == "libOriginal-Resolved.native" assert(variant.identifier == "liboriginal_resolved");

        dispose(variant);
    }

    custom_archive := B.os_load_library(paths, "Custom", "original_custom", false, static_library_suffix = ".archive");

    assert(custom_archive != null && custom_archive.full_path == "./images/original_custom");
    assert(!custom_archive.is_shared_library && B.os_symbol_exists_in_library(custom_archive, "original_static_only"));
    dispose(custom_archive);
    print("ok\n");
}
