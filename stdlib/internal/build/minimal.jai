is_absolute_path :: (path: string) -> bool {
    if path.count == 0 return false;
    return path[0] == 47 || path[0] == 92 || (path.count >= 3 && path[1] == 58);
}

build :: () {
    workspace := C.compiler_create_workspace("Target Program");
    if workspace < 0 {
        write_string("Unable to create the target workspace.\n");
        return;
    }

    parent := C.get_build_options();
    options := C.get_build_options(workspace);
    sources: [..] string;

    defer array_free(sources);
    change_directory := true;
    modified := false;

    for argument: parent.compile_time_command_line {
        if argument.count == 0 continue;

        if argument[0] != 45 {
            array_add(*sources, argument);
            continue;
        }

        if argument == "-no_cwd" change_directory = false;
        else if argument == "-release" {
            C.set_optimization(*options, .OPTIMIZED);
            options.stack_trace = false;
            modified = true;
        } else if argument == "-x64" options.backend = .X64;
        else if argument == "-llvm" options.backend = .LLVM;
    }

    original := get_working_directory();

    defer free(original);
    source_directory := original;
    owned_directory: string;

    defer if owned_directory.data free(owned_directory);

    if sources.count > 0 {
        first := sources[0];
        directory := S.path_strip_filename(first);

        options.output_executable_name = S.basename(first);

        if directory.count > 0 {
            if is_absolute_path(directory) source_directory = directory;
            else {
                owned_directory = S.join(original, "/", directory);
                source_directory = owned_directory;
            }

            options.output_path = source_directory;
            modified = true;
        }
    }

    if modified C.set_build_options(options, workspace);

    C.set_build_options_dc(.{ do_output = false, write_added_strings = false });

    if change_directory && !set_working_directory(source_directory) {
        C.compiler_report("Minimal build could not select the source working directory.");
        return;
    }

    for source: sources {
        if change_directory && !is_absolute_path(source) {
            absolute := S.join(original, "/", source);
            C.add_build_file(absolute, workspace);
            free(absolute);
        } else C.add_build_file(source, workspace);
    }
}

#scope_file
#import "Basic";
C :: #import "Compiler";
S :: #import "String";
