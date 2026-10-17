compiler_create_workspace :: (name := "") -> Workspace #compiler {
    result := __jaic_workspace_create(name);
    slot := workspace_slot(result);
    slot.name = Basic.copy_string(name);
    return result;
}

compiler_destroy_workspace :: (w: Workspace) #compiler {
    __jaic_workspace_destroy(resolve_workspace(w));
}

compiler_set_memory_breakpoint :: (pointer: *void) #compiler {
    require(false, "compiler memory breakpoints are unavailable in this compiler");
}

developer_debug :: (x: *void) #compiler {
    require(false, "developer_debug requires compiler debugger support, which is unavailable");
}

get_name :: (w: Workspace = -1) -> string #compiler {
    return workspace_slot(resolve_workspace(w)).name;
}

get_build_options :: (w: Workspace = -1) -> C.Build_Options #compiler {
    ws := resolve_workspace(w);
    result: C.Build_Options;

    if !snapshot_options(ws, *result, false) && ws == __jaic_current_workspace() {
        result.compile_time_command_line = get_toplevel_command_line();
    }

    return result;
}

set_build_options :: (options: C.Build_Options, w: Workspace = -1, loc := #caller_location) #compiler {
    // `.X64` asks for the quick-to-compile debug code generator. This compiler has a single code
    // generator, so the request is built with it; the program behaves the same either way.
    require(!options.use_natvis_compatible_types, "Natvis-compatible type emission is unavailable");
    require(options.llvm_options.command_line.count == 0, "raw LLVM command-line options are unavailable; use explicit build settings");
    require(!options.llvm_options.function_sections && !options.llvm_options.disable_mem2reg, "these LLVM pass switches are not implemented by the current backend");
    ws := resolve_workspace(w);

    require(options.os_target == .WINDOWS || options.os_target == .LINUX || options.os_target == .MACOS || options.os_target == .WASM, "jaic cannot build for `os_target = .%`", enum_text(options.os_target));
    require(options.cpu_target == .X64 || options.cpu_target == .ARM64 || options.cpu_target == .CUSTOM, "jaic cannot build for `cpu_target = .%`", enum_text(options.cpu_target));

    set_number(ws, .OUTPUT_TYPE, options.output_type);
    __jaic_workspace_set_text(ws, .OUTPUT_EXECUTABLE_NAME, options.output_executable_name);
    __jaic_workspace_set_text(ws, .OUTPUT_PATH, options.output_path);
    __jaic_workspace_set_text(ws, .INTERMEDIATE_PATH, options.intermediate_path);
    __jaic_workspace_set_text(ws, .ENTRY_POINT_NAME, options.entry_point_name);
    set_number(ws, .OS_TARGET, options.os_target);
    set_number(ws, .CPU_TARGET, options.cpu_target);
    __jaic_workspace_set_number(ws, .TEMPORARY_STORAGE_SIZE, options.temporary_storage_size);
    set_number(ws, .ARRAY_BOUNDS_CHECK, options.array_bounds_check);
    set_number(ws, .CAST_BOUNDS_CHECK, options.cast_bounds_check);
    set_number(ws, .NULL_POINTER_CHECK, options.null_pointer_check);
    set_number(ws, .ARITHMETIC_OVERFLOW_CHECK, options.arithmetic_overflow_check);
    set_number(ws, .DEAD_CODE_ELIMINATION, options.dead_code_elimination);
    set_number(ws, .RUNTIME_SUPPORT_DEFINITIONS, options.runtime_support_definitions);
    set_number(ws, .BACKTRACE_ON_CRASH, options.backtrace_on_crash);
    set_number(ws, .EMIT_DEBUG_INFO, options.emit_debug_info);
    __jaic_workspace_set_flag(ws, .STACK_TRACE, options.stack_trace);
    __jaic_workspace_set_flag(ws, .DISABLE_REDZONE, options.disable_redzone);
    __jaic_workspace_set_flag(ws, .ENABLE_FRAME_POINTERS, options.enable_frame_pointers);
    __jaic_workspace_set_flag(ws, .APPEND_EXECUTABLE_FILENAME_EXTENSION, options.append_executable_filename_extension);
    __jaic_workspace_set_flag(ws, .USE_CUSTOM_LINK_COMMAND, options.use_custom_link_command);

    if options.minimum_os_version.major || options.minimum_os_version.minor {
        __jaic_workspace_set_text(ws, .MINIMUM_OS_VERSION, Basic.sprint("%.%", options.minimum_os_version.major, options.minimum_os_version.minor));
    } else __jaic_workspace_set_text(ws, .MINIMUM_OS_VERSION, "");

    set_number(ws, .BITCODE_OPTIMIZATION, options.llvm_options.bitcode_optimization_setting);
    set_number(ws, .MACHINE_CODE_OPTIMIZATION, options.llvm_options.machine_code_optimization_setting);

    __jaic_workspace_set_text(ws, .LLVM_TARGET_SYSTEM_TRIPLE, options.llvm_options.target_system_triple);
    __jaic_workspace_set_text(ws, .LLVM_TARGET_SYSTEM_CPU, options.llvm_options.target_system_cpu);
    __jaic_workspace_set_text(ws, .LLVM_TARGET_SYSTEM_FEATURES, options.llvm_options.target_system_features);
    __jaic_workspace_set_flag(ws, .LLVM_ENABLE_TAIL_CALLS, options.llvm_options.enable_tail_calls);
    __jaic_workspace_set_flag(ws, .LLVM_ENABLE_LOOP_UNROLLING, options.llvm_options.enable_loop_unrolling);
    __jaic_workspace_set_flag(ws, .LLVM_ENABLE_SLP_VECTORIZATION, options.llvm_options.enable_slp_vectorization);
    __jaic_workspace_set_flag(ws, .LLVM_ENABLE_LOOP_VECTORIZATION, options.llvm_options.enable_loop_vectorization);
    __jaic_workspace_set_flag(ws, .LLVM_PRESERVE_DEBUG_INFO, options.llvm_options.preserve_debug_info);
    __jaic_workspace_set_flag(ws, .LLVM_MERGE_FUNCTIONS, options.llvm_options.merge_functions);
    __jaic_workspace_set_flag(ws, .LLVM_DISABLE_INLINING, options.llvm_options.disable_inlining);
    __jaic_workspace_set_flag(ws, .LLVM_DISABLE_MEM2REG, options.llvm_options.disable_mem2reg);
    __jaic_workspace_set_flag(ws, .LLVM_ENABLE_SPLIT_MODULES, options.llvm_options.enable_split_modules);
    __jaic_workspace_set_flag(ws, .LLVM_FUNCTION_SECTIONS, options.llvm_options.function_sections);
    __jaic_workspace_set_flag(ws, .OUTPUT_BITCODE, options.llvm_options.output_bitcode);
    __jaic_workspace_set_flag(ws, .OUTPUT_LLVM_IR, options.llvm_options.output_llvm_ir);
    __jaic_workspace_set_flag(ws, .OUTPUT_BITCODE_BEFORE_OPTIMIZATIONS, options.llvm_options.output_bitcode_before_optimizations);
    __jaic_workspace_set_flag(ws, .OUTPUT_LLVM_IR_BEFORE_OPTIMIZATIONS, options.llvm_options.output_llvm_ir_before_optimizations);
    __jaic_workspace_clear_list(ws, .IMPORT_PATH);
    for path: options.import_path __jaic_workspace_append_list(ws, .IMPORT_PATH, source_relative_path(path, loc));

    __jaic_workspace_clear_list(ws, .ADDITIONAL_LINKER_ARGUMENT);
    for arg: options.additional_linker_arguments __jaic_workspace_append_list(ws, .ADDITIONAL_LINKER_ARGUMENT, arg);

    snapshot_options(ws, *options, true);
}

set_build_options_dc :: (options: C.Build_Options_During_Compile, w: Workspace = -1) #compiler {
    ws := resolve_workspace(w);
    stored := get_build_options(ws);
    stored.write_added_strings = options.write_added_strings;
    stored.append_executable_filename_extension = options.append_executable_filename_extension;
    stored.interactive_bytecode_debugger = options.interactive_bytecode_debugger;
    if options.output_executable_name.count > 0 stored.output_executable_name = options.output_executable_name;
    if options.output_path.count > 0 stored.output_path = options.output_path;

    arguments: [..] string;

    defer Basic.array_free(arguments);

    if options.append_linker_arguments.count > 0 {
        for arg: stored.additional_linker_arguments Basic.array_add(*arguments, arg);

        for arg: options.append_linker_arguments {
            Basic.array_add(*arguments, arg);
            __jaic_workspace_append_list(ws, .ADDITIONAL_LINKER_ARGUMENT, arg);
        }

        stored.additional_linker_arguments = arguments;
    }

    __jaic_workspace_set_flag(ws, .DO_OUTPUT, options.do_output);
    __jaic_workspace_set_flag(ws, .APPEND_EXECUTABLE_FILENAME_EXTENSION, options.append_executable_filename_extension);
    __jaic_workspace_set_text(ws, .OUTPUT_EXECUTABLE_NAME, stored.output_executable_name);
    __jaic_workspace_set_text(ws, .OUTPUT_PATH, stored.output_path);
    snapshot_options(ws, *stored, true);
}

add_build_file :: (filename: string, w: Workspace, loc := #caller_location) #compiler {
    __jaic_workspace_add_file(resolve_workspace(w), source_relative_path(filename, loc));
}

add_build_string :: (data: string, w: Workspace, code := #code,null, loc := #caller_location) #compiler "add_build_string_scoped_by_code" {
    ws := resolve_workspace(w);
    if __jaic_code_is_null(code) __jaic_workspace_add_string(ws, data);
    else __jaic_workspace_add_string_to_code(ws, data, code);
}

add_build_string :: (data: string, w: Workspace, message: *C.Message, loc := #caller_location) #compiler "add_build_string_scoped_by_message" {
    ws := resolve_workspace(w);

    if !message {
        __jaic_workspace_add_string(ws, data);
        return;
    }

    require(message.workspace == ws, "add_build_string message belongs to another workspace");
    require(message.kind == .FILE || message.kind == .IMPORT, "add_build_string requires FILE or IMPORT message");
    __jaic_workspace_add_string_to_module(ws, data, record_id(message));
}

compiler_report :: (message: string, loc := #caller_location, mode := C.Report.ERROR) #compiler {
    __jaic_report(message, loc.fully_pathed_filename, loc.line_number, loc.character_number, cast(u8) mode);
}

compiler_report :: (filename: string, line_number: s64, character_number: s64, error_text: string, mode := C.Report.ERROR) {
    __jaic_report(error_text, filename, line_number, character_number, cast(u8) mode);
}

compiler_get_version_info :: (version_info_return: *C.Version_Info) -> string #compiler {
    result: string;

    __jaic_compiler_version(*result);

    if version_info_return {
        values: [3] s32;
        index: s64 = 0;
        active := false;

        for byte: result {
            if byte >= #char "0" && byte <= #char "9" {
                if index < 3 values[index] = values[index] * 10 + cast(s32) (byte - #char "0");
                active = true;
            } else if active {
                index += 1;
                active = false;
                if index == 3 break;
            }
        }

        version_info_return.* = .{ values[0], values[1], values[2] };
    }

    return result;
}

get_toplevel_command_line :: () -> [] string #compiler {
    result: [..] string;
    count := __jaic_command_line_count();

    for i: 0..count - 1 {
        value: string;
        __jaic_command_line_arg(i, *value);
        Basic.array_add(*result, value);
    }

    return result;
}

compiler_get_base_path :: () -> string #compiler {
    result: string;
    __jaic_base_path(*result);
    return result;
}

compiler_add_library_search_directory :: (path: string) #compiler {
    __jaic_add_library_dir(path);
}

compiler_set_type_info_flags :: (type: Type, flags: C.Type_Info_Flags) #compiler {
    __jaic_set_type_info_flags(type, cast(u32) flags);
}

compiler_get_struct_location :: (w: Workspace, info: *Type_Info_Struct) -> Source_Code_Location #compiler {
    require(resolve_workspace(w) == __jaic_current_workspace(), "foreign workspace struct locations require matching descriptors");
    result: Source_Code_Location;
    __jaic_struct_location(info, *result.fully_pathed_filename, *result.line_number, *result.character_number);
    return result;
}

remap_import :: (w: Workspace, host_module_name: string, import_name: string, replacement_name: string) #compiler {
    __jaic_workspace_remap_import(resolve_workspace(w), host_module_name, import_name, replacement_name);
}

provide_import :: (w: Workspace, message: *C.Message_Failed_Import, type: C.Provided_Import_Type, value: string) #compiler {
    require(message != null, "provide_import requires a failed import message");
    __jaic_workspace_provide_import(resolve_workspace(w), message.host_module_name, message.target_module_name, cast(u8) type, value);
}

#scope_module
snapshot_options :: (w: Workspace, options: *C.Build_Options, save: bool) -> bool {
    strings: [..] s64;
    arrays: [..] s64;

    defer Basic.array_free(strings);
    defer Basic.array_free(arrays);
    option_view_offsets(type_info(C.Build_Options), 0, *strings, *arrays);

    return __jaic_workspace_options_snapshot(w, options, size_of(C.Build_Options), strings, arrays, save);
}

option_view_offsets :: (info: *Type_Info, offset: s64, strings: *[..] s64, arrays: *[..] s64) {
    if info.type == .STRING {
        Basic.array_add(strings, offset);
    } else if info.type == .STRUCT {
        structure := cast(*Type_Info_Struct) info;
        for member: structure.members {
            if member.flags & .CONSTANT || member.offset_in_bytes < 0 continue;
            option_view_offsets(member.type, offset + member.offset_in_bytes, strings, arrays);
        }
    } else if info.type == .ARRAY {
        array := cast(*Type_Info_Array) info;
        if array.array_type == .FIXED {
            for i: 0..array.array_count - 1 option_view_offsets(array.element_type, offset + i * array.element_type.runtime_size, strings, arrays);
        } else {
            require(array.array_type == .VIEW && array.element_type.type == .STRING, "unsupported build option view schema");
            Basic.array_add(arrays, offset);
        }
    }
}

Workspace_State :: struct {
    name: string;
    intercepting: bool;
    finished: bool;
    intercept_flags: C.Intercept_Flags;
    link_phase: *C.Message_Phase;
}

states: [..] *Workspace_State;

resolve_workspace :: (w: Workspace) -> Workspace {
    if w == -1 return __jaic_current_workspace();
    return w;
}

workspace_slot :: (w: Workspace) -> *Workspace_State {
    __jaic_workspace_validate(w);
    require(w >= 0 && w < 1000000, "invalid workspace %", w);
    while states.count <= w Basic.array_add(*states, cast(*Workspace_State) null);

    if !states[w] {
        states[w] = Basic.New(Workspace_State);
    }

    return states[w];
}

enum_text :: (value: $T) -> string {
    info := cast(*Type_Info_Enum) type_info(T);
    for number, i: info.values if number == cast(s64) value return info.names[i];
    require(false, "unrecognized compiler enum value");
    return "";
}

append_linker_argument :: (w: Workspace, argument: string) {
    __jaic_workspace_append_list(w, .ADDITIONAL_LINKER_ARGUMENT, argument);
}

set_number :: (w: Workspace, option: Workspace_Number, value: $T) {
    __jaic_workspace_set_number(w, option, cast(s64) value);
}

source_relative_path :: (path: string, loc: Source_Code_Location) -> string {
    if path.count == 0 || path[0] == #char "/" || path[0] == #char "\\" || (path.count > 1 && path[1] == #char ":") return path;
    _, directory := get_basename_and_path(loc.fully_pathed_filename);
    return Basic.sprint("%%", directory, path);
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
