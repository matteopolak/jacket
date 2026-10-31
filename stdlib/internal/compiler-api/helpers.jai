// Sets the options that go with an optimization level: bounds and cast checks, the LLVM optimization settings, inlining and so on. `preserve_debug_info` keeps or drops debug information.
//
// ```jai
// options: Build_Options;
// set_optimization(*options, .OPTIMIZED);
// assert(options.array_bounds_check == .OFF);
// set_optimization(*options, .DEBUG);
// assert(options.array_bounds_check == .ON);
// ```
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

// Copies the options that a workspace passes on to the workspaces it creates (`Build_Options.Commonly_Propagated`) from `a` to `b`.
copy_commonly_propagated_fields :: (a: C.Build_Options, b: *C.Build_Options) {
    b.Commonly_Propagated = a.Commonly_Propagated;
}

// Allocates a number literal node with the value `v`, for building code by hand.
//
// ```jai
// literal := make_integer_literal(42);
// assert(literal._s64 == 42 && literal.kind == .LITERAL);
// ```
make_integer_literal :: (v: s64) -> *C.Code_Literal {
    result := Basic.New(C.Code_Literal);
    result.kind = .LITERAL;
    result.value_type = .NUMBER;
    result.value_flags = .IS_A_NUMBER;
    result._s64 = v;
    if v < 0 result.value_flags |= .MINUS_SIGN;

    return result;
}

// Allocates a string literal node holding a copy of `s`.
//
// ```jai
// literal := make_string_literal("hi");
// assert(literal._string == "hi");
// ```
make_string_literal :: (s: string) -> *C.Code_Literal {
    result := Basic.New(C.Code_Literal);
    result.kind = .LITERAL;
    result.value_type = .STRING;
    result._string = Basic.copy_string(s);

    return result;
}

// Returns the file, line and column where `c` was written, as a `Source_Code_Location`.
make_location :: (c: *C.Code_Node) -> Source_Code_Location {
    if !c return .{};
    return .{ get_filename(c), c.l0, c.c0 };
}

// Returns the full path of the file `node` was loaded from, or an empty string.
get_filename :: (node: *C.Code_Node) -> string {
    if !node || !node.enclosing_load return "";
    return node.enclosing_load.fully_pathed_filename;
}

// Splits a file path into its name without the extension and its folder (with the trailing slash).
//
// ```jai
// name, folder := get_basename_and_path("src/lib/parser.jai");
// assert(name == "parser" && folder == "src/lib/");
// ```
get_basename_and_path :: (input_name: string) -> (basename: string, path: string) {
    split: s64 = -1;
    for c, i: input_name if c == #char "/" || c == #char "\\" split = i;

    name := string.{ input_name.count - split - 1, input_name.data + split + 1 };
    path := string.{ split + 1, input_name.data };

    // The extension is dropped; a leading dot is part of the name, not an extension.
    for < index: 1..name.count - 1 {
        if name[index] == #char "." {
            name.count = index;
            break;
        }
    }

    return name, path;
}

// True if the struct `info` is, or starts with a `#as using` member that is, a struct called `base_name`.
is_subclass_of :: (info: *Type_Info, base_name: string, first_call := true) -> bool {
    Basic.assert(first_call, "is_subclass_of(first_call=false) requires a behavioral contract");
    return has_base_type(info, base_name);
}

// Returns the source spelling of an operator, such as `"=="` or `"<<"`.
//
// ```jai
// assert(operator_to_string(.IS_EQUAL) == "==");
// ```
operator_to_string :: (op: C.Operator_Type) -> string {
    return operator_text(cast(s32) op);
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
