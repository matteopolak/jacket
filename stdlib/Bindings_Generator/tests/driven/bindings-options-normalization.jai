#import "Basic";
#import "String";
FS :: #import "File_Utilities";
B :: #import "Bindings_Generator";

warnings, errors: int;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .WARNING warnings += 1;
    if info.common_flags & .ERROR errors += 1;
}

release_array :: (original: [..] $T, normalized: [..] T) {
    if normalized.data != original.data array_free(normalized);
}

release :: (original: B.Generate_Bindings_Options, normalized: B.Generate_Bindings_Options) {
    release_array(original.libraries, normalized.libraries);
    release_array(original.system_libraries, normalized.system_libraries);
    release_array(original.library_search_paths, normalized.library_search_paths);
    release_array(original.system_library_search_paths, normalized.system_library_search_paths);
    release_array(original.system_include_paths, normalized.system_include_paths);
    release_array(original.extra_clang_arguments, normalized.extra_clang_arguments);

    for text, index: normalized.path_fragments_to_treat_as_non_system_paths {
        if text.data != original.path_fragments_to_treat_as_non_system_paths[index].data free(text);
    }

    release_array(original.path_fragments_to_treat_as_non_system_paths, normalized.path_fragments_to_treat_as_non_system_paths);
}

main :: () {
    #if OS == .MACOS {
        defaults: B.Generate_Bindings_Options;
        native_defaults, defaults_ok := B.normalize_options(defaults);

        defer release(defaults, native_defaults);
        defer for path: native_defaults.system_include_paths free(path);
        defer for path: native_defaults.system_library_search_paths free(path);
        defer if native_defaults.extra_clang_arguments.count == 2 free(native_defaults.extra_clang_arguments[1]);
        assert(defaults_ok && native_defaults.libraries.count == 0 && native_defaults.system_libraries.count == 0);
        assert(native_defaults.system_include_paths.count > 0 && native_defaults.system_library_search_paths.count == 1);
        for path: native_defaults.system_include_paths assert(FS.file_exists(path));

        assert(ends_with(native_defaults.system_library_search_paths[0], "/usr/lib") && FS.file_exists(native_defaults.system_library_search_paths[0]));
        expected := ifx defaults.cpu == .ARM64 then "arm64-apple-darwin" else "x86_64-apple-darwin";

        assert(native_defaults.extra_clang_arguments.count == 2 && native_defaults.extra_clang_arguments[0] == "-target" && native_defaults.extra_clang_arguments[1] == expected);
    }

    // Explicit canonical native paths isolate option semantics from discovery.
    options: B.Generate_Bindings_Options;

    array_add(*options.system_include_paths, "owned-include-root");
    array_add(*options.system_library_search_paths, "owned-system-root");
    defer array_free(options.system_include_paths);
    defer array_free(options.system_library_search_paths);
    zero, accepted := B.normalize_options(options);

    defer release(options, zero);
    assert(accepted && zero.libraries.count == 0 && zero.system_libraries.count == 0);
    assert(zero.library_search_paths.count == 1 && zero.library_search_paths[0] == ".");
    assert(zero.static_library_suffix == ".a" && zero.shared_library_suffix == "");
    assert(zero.get_func_args_for_printing == options.get_func_args_for_printing);
    assert(zero.source_files.count == 0);
    mobile := options;
    mobile.os = .ANDROID;
    mobile.cpu = .ARM64;
    android, mobile_accepted := B.normalize_options(mobile);

    defer release(mobile, android);
    assert(mobile_accepted && android.static_library_suffix == ".a" && android.shared_library_suffix == ".so");
    assert(android.extra_clang_arguments.count == 2 && android.extra_clang_arguments[0] == "-target" && android.extra_clang_arguments[1] == "aarch64-linux-android");

    array_add(*options.libraries, B.Library.{ "ordinary-first", "ordinary" });
    array_add(*options.system_libraries, B.Library.{ "system-first", "system" });
    array_add(*options.library_search_paths, "ordinary-root");
    array_add(*options.libnames, "legacy-one", "legacy-one");
    array_add(*options.system_library_names, "legacy-system");
    array_add(*options.libpaths, "legacy-root");
    array_add(*options.system_library_paths, "legacy-system-root");
    array_add(*options.extra_clang_arguments, "-target", "explicit-target");
    mixed := copy_string("MiXeD/Path");

    defer free(mixed);
    array_add(*options.path_fragments_to_treat_as_non_system_paths, mixed);
    defer array_free(options.libraries);
    defer array_free(options.system_libraries);
    defer array_free(options.library_search_paths);
    defer array_free(options.libnames);
    defer array_free(options.system_library_names);
    defer array_free(options.libpaths);
    defer array_free(options.system_library_paths);
    defer array_free(options.extra_clang_arguments);
    defer array_free(options.path_fragments_to_treat_as_non_system_paths);
    options.omit_global_declarations = true;
    options.c_enum_emulation = true;
    options.generate_bitfield_helpers = false;
    options.get_func_args_for_printing = null;
    normalized: B.Generate_Bindings_Options;

    push_context {
        context.logger = capture;
        normalized, accepted = B.normalize_options(options);
    }

    defer release(options, normalized);
    assert(accepted && warnings == 4 && errors == 0);
    assert(normalized.libraries.count == 3 && normalized.system_libraries.count == 2);
    assert(normalized.libraries[0].filename == "ordinary-first" && normalized.libraries[0].identifier == "ordinary");
    assert(normalized.libraries[1].filename == "legacy-one" && normalized.libraries[1].identifier == "");
    assert(normalized.libraries[2].filename == "legacy-one" && normalized.libraries[2].identifier == "");
    assert(normalized.system_libraries[0].filename == "system-first" && normalized.system_libraries[1].filename == "legacy-system");
    assert(normalized.library_search_paths.count == 2 && normalized.library_search_paths[0] == "ordinary-root" && normalized.library_search_paths[1] == "legacy-root");
    assert(normalized.system_library_search_paths.count == 2 && normalized.system_library_search_paths[0] == "owned-system-root" && normalized.system_library_search_paths[1] == "legacy-system-root");
    assert(normalized.libnames.count == 0 && normalized.libpaths.count == 0 && normalized.system_library_names.count == 0 && normalized.system_library_paths.count == 0);
    assert(normalized.extra_clang_arguments.count == 2 && normalized.extra_clang_arguments[1] == "explicit-target");
    assert(normalized.path_fragments_to_treat_as_non_system_paths[0] == "mixed/path");
    assert(normalized.omit_global_declarations && normalized.c_enum_emulation && !normalized.generate_bitfield_helpers);
    assert(normalized.get_func_args_for_printing == null);
    assert(options.libraries.count == 1 && options.libnames.count == 2);
    print("ok\n");
}
