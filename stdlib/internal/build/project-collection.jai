#scope_module

pg_reset :: () {
    for path: Project_Generator.project_files free(path);
    array_reset(*Project_Generator.project_files);
}

pg_add :: (file: string, take_ownership: bool) {
    for path: Project_Generator.project_files if path == file {
        if take_ownership free(file);
        return;
    }

    retained := file;
    if !take_ownership retained = copy_string(file);

    array_add(*Project_Generator.project_files, retained);
}

pg_observed_executable: string;

pg_message :: (message: *PG_Compiler.Message) {
    if !message {
        pg_fail("Project generator received a null compiler message");
        return;
    }

    if message.kind == .FILE {
        file := cast(*PG_Compiler.Message_File) message;
        if !file.from_a_string pg_add(file.fully_pathed_filename, false);
    } else if message.kind == .PHASE {
        phase := cast(*PG_Compiler.Message_Phase) message;
        if phase.phase == .PRE_WRITE_EXECUTABLE {
            free(pg_observed_executable);
            pg_observed_executable = copy_string(phase.executable_name);
        }
    } else if message.kind == .COMPLETE {
        options := PG_Compiler.get_build_options(message.workspace);
        executable := options.output_executable_name;

        if executable.count == 0 executable = pg_observed_executable;

        if Project_Generator.project_name.count == 0 {
            if executable.count == 0 {
                pg_fail("Project generator has no target executable name");
                return;
            }

            Project_Generator.project_name = copy_string(PG_String.basename(executable));
        }

        if Project_Generator.debug_command.count == 0 Project_Generator.debug_command = PG_String.join(PG_String.basename(executable), ".exe");

        if Project_Generator.working_directory.count == 0 {
            path := options.output_path;
            if path.count == 0 path = PG_String.path_strip_filename(pg_observed_executable);
            Project_Generator.working_directory = copy_string(path);
        }

        pg_add(PG_String.join(Project_Generator.module_path, "/../editor_support/msvc/jai.natvis"), true);
        pg_emit();
        pg_reset();
        free(pg_observed_executable);
        pg_observed_executable = "";
    }
}

pg_build :: (file: string) {
    current := PG_Compiler.get_build_options();
    current.output_type = .NO_OUTPUT;
    PG_Compiler.set_build_options(current);
    target := PG_Compiler.compiler_create_workspace("Project generator target");
    if target < 0 {
        pg_fail("Project generator could not create a workspace");
        return;
    }

    absolute, success := PG_File.get_absolute_path(file);
    if !success {
        pg_fail("Project generator source path could not be resolved");
        return;
    }

    defer free(absolute);
    options := PG_Compiler.get_build_options(target);

    PG_Compiler.copy_commonly_propagated_fields(current, *options);
    options.output_type = .EXECUTABLE;
    options.output_executable_name = PG_String.basename(absolute);
    options.output_path = PG_String.path_strip_filename(absolute);
    PG_Compiler.set_build_options(options, target);
    PG_Compiler.compiler_begin_intercept(target);
    PG_Compiler.add_build_file(absolute, target);
    pg_intercept(target);
}

pg_intercept :: (w: Workspace) {
    while true {
        message := PG_Compiler.compiler_wait_for_message();
        if !message {
            PG_Compiler.compiler_end_intercept(w);
            pg_fail("Project generator workspace ended without COMPLETE");
            return;
        }

        if message.workspace != w continue;

        pg_message(message);

        if message.kind == .COMPLETE {
            PG_Compiler.compiler_end_intercept(w);
            complete := cast(*PG_Compiler.Message_Complete) message;
            if complete.error_code != .NONE PG_Compiler.compiler_set_workspace_status(.FAILED, w);
            return;
        }
    }
}

pg_fail :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
}
