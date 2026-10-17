set_optimization :: (using build_options: *C.Build_Options, type: C.Optimization_Type, preserve_debug_info := true) {
    debug := type == .DEBUG || type == .VERY_DEBUG;
    if debug {
        array_bounds_check = .ON;
        cast_bounds_check = .FATAL;
        null_pointer_check = .ON;
        llvm_options.bitcode_optimization_setting = .O0;
        llvm_options.machine_code_optimization_setting = .NONE;
    } else {
        array_bounds_check = .OFF;
        cast_bounds_check = .OFF;
        null_pointer_check = .OFF;

        if type == .OPTIMIZED {
            llvm_options.bitcode_optimization_setting = .O2;
            llvm_options.machine_code_optimization_setting = .DEFAULT;
        } else if type == .VERY_OPTIMIZED {
            llvm_options.bitcode_optimization_setting = .O3;
            llvm_options.machine_code_optimization_setting = .AGGRESSIVE;
        } else if type == .OPTIMIZED_SMALL {
            llvm_options.bitcode_optimization_setting = .OS;
            llvm_options.machine_code_optimization_setting = .DEFAULT;
        } else {
            llvm_options.bitcode_optimization_setting = .OZ;
            llvm_options.machine_code_optimization_setting = .AGGRESSIVE;
        }
    }

    llvm_options.disable_inlining = debug;
    llvm_options.enable_tail_calls = !debug;
    llvm_options.enable_loop_unrolling = !debug;
    llvm_options.enable_slp_vectorization = type == .VERY_OPTIMIZED;
    llvm_options.enable_loop_vectorization = type == .VERY_OPTIMIZED;
    llvm_options.enable_split_modules = debug;
    x64_options.enable_unix_runtime_frame_information = debug;
    enable_frame_pointers = os_target == .MACOS || debug || preserve_debug_info;
    llvm_options.preserve_debug_info = preserve_debug_info;
    if !preserve_debug_info emit_debug_info = .NONE;
}

copy_commonly_propagated_fields :: (a: C.Build_Options, b: *C.Build_Options) {
    b.Commonly_Propagated = a.Commonly_Propagated;
}

make_integer_literal :: (v: s64) -> *C.Code_Literal {
    result := Basic.New(C.Code_Literal);
    result.kind = .LITERAL;
    result.value_type = .NUMBER;
    result.value_flags = .IS_A_NUMBER;
    result._s64 = v;
    if v < 0 result.value_flags |= .MINUS_SIGN;

    return result;
}

make_string_literal :: (s: string) -> *C.Code_Literal {
    result := Basic.New(C.Code_Literal);
    result.kind = .LITERAL;
    result.value_type = .STRING;
    result._string = Basic.copy_string(s);

    return result;
}

make_location :: (c: *C.Code_Node) -> Source_Code_Location {
    if !c return .{};
    return .{ get_filename(c), c.l0, c.c0 };
}

get_filename :: (node: *C.Code_Node) -> string {
    if !node || !node.enclosing_load return "";
    return node.enclosing_load.fully_pathed_filename;
}

get_basename_and_path :: (input_name: string) -> (basename: string, path: string) {
    split: s64 = -1;
    for c, i: input_name if c == #char "/" || c == #char "\\" split = i;
    if split < 0 return input_name, "";
    return .{ input_name.count - split - 1, input_name.data + split + 1 }, .{ split + 1, input_name.data };
}

is_subclass_of :: (info: *Type_Info, base_name: string, first_call := true) -> bool {
    require(first_call, "is_subclass_of(first_call=false) requires a behavioral contract");
    return has_base_type(info, base_name);
}

operator_to_string :: (op: C.Operator_Type) -> string {
    return operator_text(cast(s32) op);
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
