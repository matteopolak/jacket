#scope_module

shutdown_plugins :: () {
    for p: plugins if p.shutdown p.shutdown(p);
    array_reset(*plugins);
}

execute_default_build :: () {
    configure_parent_output(false);
    w := C.compiler_create_workspace("Default build target");
    if w < 0 {
        print("Unable to create the target workspace.\n");
        return;
    }

    arguments := C.get_toplevel_command_line();

    defer array_free(arguments);
    request := parse_build_arguments(arguments, C.get_build_options(w));

    defer free_build_request(*request);

    if !initialize_plugins(w, *request) {
        print("Plugin initialization failed.\n");
        shutdown_plugins();
        exit(0);
    }

    if request.help {
        write_string(HELP_STRING);
        for p: plugins if p.log_help p.log_help(p);
    }

    if request.version print("%\n", C.compiler_get_version_info(null));

    if request.sources.count == 0 && request.snippets.count == 0 {
        shutdown_plugins();
        C.compiler_destroy_workspace(w);
        if request.help || request.version return;
        option_error("Provide a source file, -add or -run input.");
    }

    directory := get_working_directory();

    defer free(directory);
    source_directory := directory;
    absolute_source: string;

    if request.sources.count > 0 {
        success: bool;
        absolute_source, success = F.get_absolute_path(request.sources[0]);
        if !success {
            shutdown_plugins();
            option_error("Cannot resolve the first source path.");
        }

        source_directory = S.path_strip_filename(absolute_source);
        if !request.explicit_exe request.options.output_executable_name = S.basename(absolute_source);
        if !request.explicit_path request.options.output_path = source_directory;
    }

    defer if absolute_source.data free(absolute_source);
    paths: [..] string;
    owned_paths: [..] string;

    defer {
        for path: owned_paths free(path);
        array_free(owned_paths);
        array_free(paths);
    }

    for path: request.import_paths {
        resolved := path;

        if !is_absolute_path(path) {
            resolved = S.join(source_directory, "/", path);
            array_add(*owned_paths, resolved);
        }

        array_add(*paths, resolved);
    }

    for path: request.options.import_path array_add(*paths, path);

    request.options.import_path = paths;
    request.options.compile_time_command_line = request.target_arguments;
    if request.options.interactive_bytecode_debugger {
        configure_parent_output(true);
    }

    if verbose {
        print("Default build arguments: %\n", arguments);
        print("Working directory: %\nOutput: %/%\n", directory, request.options.output_path, request.options.output_executable_name);
    }

    C.set_build_options(request.options, w);
    flags: C.Intercept_Flags;
    for p: plugins if p.before_intercept p.before_intercept(p, *flags);

    C.compiler_begin_intercept(w, flags);

    for source: request.sources {
        absolute, success := F.get_absolute_path(source);
        if !success {
            C.compiler_end_intercept(w);
            shutdown_plugins();
            option_error(sprint("Cannot resolve source '%'.", source));
        }

        C.add_build_file(absolute, w);
        free(absolute);
    }

    for code: request.snippets C.add_build_string(code, w);
    for p: plugins if p.add_source p.add_source(p);

    message_loop(w);
}

configure_parent_output :: (debugger: bool) {
    parent := C.get_build_options();
    during: C.Build_Options_During_Compile;
    during.do_output = false;
    during.write_added_strings = parent.write_added_strings;
    during.append_executable_filename_extension = parent.append_executable_filename_extension;
    during.interactive_bytecode_debugger = debugger || parent.interactive_bytecode_debugger;
    C.set_build_options_dc(during);
}
