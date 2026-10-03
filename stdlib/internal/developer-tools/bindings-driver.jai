_default_get_func_args_for_printing :: (func: *Function, type: *Function_Type, mode: Arg_Print_Mode) -> [] *Declaration {
    assert(mode == .OUTER_WRAPPER || mode == .FOREIGN_FUNC_DECLARATION || mode == .INNER_WRAPPER_CALL, "unknown argument print mode");
    if !type && func && func.type type = func.type.type_of_function;
    if !type return .{};
    return type.arguments;
}

#load "binding-options-normalization.jai";

normalize_options :: (original_options: Generate_Bindings_Options) -> Generate_Bindings_Options, bool {
    state: Binding_Options_Normalization;
    state.options = original_options;
    accepted := binding_normalize_options(*state);

    // Returned arrays/text belong to the caller allocator. Only the private
    // ownership descriptor is released here; high-level generation keeps it.
    array_free(state.owned_text);

    return state.options, accepted;
}

generate_bindings :: (_options: Generate_Bindings_Options, log_results := false) -> generated_result: String_Builder, success: bool {
    return binding_output(_options, log_results, false);
}

generate_bridge :: (_options: Generate_Bindings_Options, filename: string) -> success: bool {
    result, ok := binding_output(_options, false, true);

    defer free_buffers(*result);
    if !ok return false;

    if !write_entire_file(filename, *result, false) {
        log_error("Bindings_Generator: could not write C++ bridge %\n", filename);
        return false;
    }

    return true;
}

#scope_file
#load "binding-canonical-driver.jai";

binding_output :: (_options: Generate_Bindings_Options, log_results: bool, bridge: bool) -> generated_result: String_Builder, success: bool {
    return binding_canonical_output(_options, log_results, bridge);
}

#scope_export
generate_bindings :: (options: Generate_Bindings_Options, filename: string, log_results := true) -> success: bool {
    result, ok := generate_bindings(options, log_results = log_results);

    defer free_buffers(*result);
    if !ok return false;

    if !write_entire_file(filename, *result, false) {
        log_error("Bindings_Generator: could not write %\n", filename);
        return false;
    }

    return true;
}

looks_like_valid_identifier :: (identifier: string) -> bool {
    if !identifier return false;

    for c, index: identifier {
        if (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_" continue;
        if index > 0 && c >= #char "0" && c <= #char "9" continue;
        return false;
    }

    return true;
}

get_default_system_include_paths :: (os := OS, cpu := CPU) -> [] string {
    result: [..] string;
    triple := binding_target_triple(os, cpu);

    if !triple {
        log_error("Bindings_Generator: unsupported include-probe target\n");
        return .{};
    }

    compiler_path, compiler_path_ok := read_entire_file(tprint("%clang-bindings/compiler-path.txt", #filepath));

    defer free(compiler_path);

    if !compiler_path_ok || !compiler_path || S.contains(compiler_path, "\0") {
        log_error("Bindings_Generator: missing or invalid source-built SDK compiler configuration\n");
        return .{};
    }

    status, output, errors, timed_out := P.run_command(compiler_path, "-E", "-x", "c", "-", "-v", tprint("--target=%", triple), capture_and_return_output = true);

    defer free(output);
    defer free(errors);

    if status.type != .EXITED || status.exit_code != 0 || timed_out {
        log_error("Bindings_Generator: C compiler include-path probe failed\n%", errors);
        return .{};
    }

    lines := S.split(errors, "\n");

    defer free(lines.data);
    active := false;

    for line: lines {
        if line == "#include <...> search starts here:" {
            active = true;
            continue;
        }

        if line == "End of search list." break;
        if !active continue;
        while line.count > 0 && line[0] == #char " " line = S.slice(line, 1, line.count - 1);

        suffix := " (framework directory)";
        if line.count >= suffix.count && S.slice(line, line.count - suffix.count, suffix.count) == suffix continue;
        if line.count > 0 array_add(*result, copy_string(line));
    }

    if !active log_error("Bindings_Generator: C compiler did not provide an include search list\n");

    return .{ result.count, result.data };
}

#scope_module
#import "Basic";
#import "File";
P :: #import "Process";
S :: #import "String";

binding_builder_text :: (builder: *String_Builder) -> string {
    push_allocator(temp);
    return builder_to_string(builder, do_reset = false);
}

binding_join_names :: (names: [] string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);

    for name, index: names {
        if index > 0 append(*builder, ",");
        append(*builder, name);
    }

    return binding_builder_text(*builder);
}

binding_target_triple :: (os: Operating_System_Tag, cpu: CPU_Tag) -> string {
    options: Generate_Bindings_Options;
    options.os = os;
    options.cpu = cpu;
    return gm_lifecycle_target(options);
}
