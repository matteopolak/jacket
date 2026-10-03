#scope_module
#import "Basic";

Binding_Options_Normalization :: struct {
    options: Generate_Bindings_Options;
    owns_arrays: bool;
    owned_text: [..] string;
}

binding_copy_option_array :: (source: [] $T) -> [..] T {
    result: [..] T;
    for item: source array_add(*result, item);
    return result;
}

binding_normalization_text :: (state: *Binding_Options_Normalization, text: string) -> string {
    owned := copy_string(text);
    array_add(*state.owned_text, owned);
    return owned;
}

binding_release_normalized_options :: (state: *Binding_Options_Normalization) {
    if state.owns_arrays {
        array_free(state.options.libraries);
        array_free(state.options.system_libraries);
        array_free(state.options.library_search_paths);
        array_free(state.options.system_library_search_paths);
        array_free(state.options.system_include_paths);
        array_free(state.options.extra_clang_arguments);
        array_free(state.options.path_fragments_to_treat_as_non_system_paths);
    }

    for text: state.owned_text free(text);

    array_free(state.owned_text);
    state.* = .{};
}

binding_normalization_library_root :: (headers: string) -> string {
    marker := "/usr/include";
    for index: 0..headers.count - marker.count {
        if string.{ marker.count, headers.data + index } != marker continue;
        before := string.{ index, headers.data };
        after := string.{ headers.count - index - marker.count, headers.data + index + marker.count };
        return sprint("%/usr/lib%", before, after);
    }

    return "";
}

binding_normalize_options :: (state: *Binding_Options_Normalization) -> bool {
    options := *state.options;
    triple := gm_lifecycle_target(options.*);

    if !triple {
        log_error("Bindings_Generator: option normalization has no supported SDK target for OS % CPU %\n", options.os, options.cpu);
        return false;
    }

    options.libraries = binding_copy_option_array(options.libraries);
    options.system_libraries = binding_copy_option_array(options.system_libraries);
    options.library_search_paths = binding_copy_option_array(options.library_search_paths);
    options.system_library_search_paths = binding_copy_option_array(options.system_library_search_paths);
    options.system_include_paths = binding_copy_option_array(options.system_include_paths);
    options.extra_clang_arguments = binding_copy_option_array(options.extra_clang_arguments);
    options.path_fragments_to_treat_as_non_system_paths = binding_copy_option_array(options.path_fragments_to_treat_as_non_system_paths);
    state.owns_arrays = true;

    if options.system_include_paths.count == 0 {
        defaults := get_default_system_include_paths(options.os, options.cpu);

        defer array_free(defaults);

        for path: defaults {
            array_add(*options.system_include_paths, path);
            array_add(*state.owned_text, path);
        }
    }

    if options.os != .WINDOWS && !options.static_library_suffix options.static_library_suffix = ".a";
    if options.os == .ANDROID && !options.shared_library_suffix options.shared_library_suffix = ".so";

    needs_system_roots := options.system_library_search_paths.count == 0;
    if needs_system_roots && (options.os == .MACOS || options.os == .IOS || options.os == .ANDROID) {
        generic, platform, found := get_libc_paths(options.os, options.cpu);
        if !found return false;

        defer free(generic);
        defer free(platform);
        root := binding_normalization_library_root(generic);
        if !root {
            log_error("Bindings_Generator: SDK include root has no corresponding library root\n");
            return false;
        }

        array_add(*options.system_library_search_paths, root);
        array_add(*state.owned_text, root);

        if options.os == .ANDROID && platform != generic {
            target_root := binding_normalization_library_root(platform);
            if !target_root {
                log_error("Bindings_Generator: target SDK include root has no corresponding library root\n");
                return false;
            }

            array_add(*options.system_library_search_paths, target_root);
            array_add(*state.owned_text, target_root);
        }
    }

    if options.os == .ANDROID || (needs_system_roots && (options.os == .MACOS || options.os == .IOS)) {
        explicit_target := false;
        for argument: options.extra_clang_arguments if argument == "-target" explicit_target = true;

        if !explicit_target {
            array_add(*options.extra_clang_arguments, "-target");
            array_add(*options.extra_clang_arguments, binding_normalization_text(state, triple));
        }
    }

    if options.libnames.count {
        log("Bindings_Generator: libnames is deprecated; use libraries\n", flags = .WARNING);
        for name: options.libnames array_add(*options.libraries, Library.{ name, "" });
        options.libnames = .{};
    }

    if options.system_library_names.count {
        log("Bindings_Generator: system_library_names is deprecated; use system_libraries\n", flags = .WARNING);
        for name: options.system_library_names array_add(*options.system_libraries, Library.{ name, "" });
        options.system_library_names = .{};
    }

    if options.libpaths.count {
        log("Bindings_Generator: libpaths is deprecated; use library_search_paths\n", flags = .WARNING);
        for path: options.libpaths array_add(*options.library_search_paths, path);
        options.libpaths = .{};
    }

    if options.system_library_paths.count {
        log("Bindings_Generator: system_library_paths is deprecated; use system_library_search_paths\n", flags = .WARNING);
        for path: options.system_library_paths array_add(*options.system_library_search_paths, path);
        options.system_library_paths = .{};
    }

    if options.library_search_paths.count == 0 array_add(*options.library_search_paths, ".");

    for *fragment: options.path_fragments_to_treat_as_non_system_paths {
        lowered := binding_normalization_text(state, fragment.*);
        for byte, index: lowered if byte >= #char "A" && byte <= #char "Z" lowered.data[index] = byte + (#char "a" - #char "A");
        fragment.* = lowered;
    }

    return true;
}
