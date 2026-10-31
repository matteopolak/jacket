// A data segment added to the built program with [add_data_segment]. Pass it back as `user_segment` to [add_global_data].
Data_Segment :: struct {}

// What a custom data segment allows: reading, writing, executing, and whether it starts zeroed.
Data_Segment_Characteristics :: enum_flags u32 {
    READ :: 0x1;
    WRITE :: 0x2;
    EXECUTE :: 0x4;
    ZEROED :: 0x8;
}

// Which segment [add_global_data] puts data in: one of the built-in ones, or `USER_SEGMENT` for one you added.
Data_Segment_Index :: enum u16 {
    WRITABLE :: 0x0;
    WRITABLE_NO_RESET :: 0x1;
    READ_ONLY :: 0x2;
    BSS :: 0x3;
    USER_SEGMENT :: 0x10;
}

// The global data of a built program, as a list of segments. Part of [Runtime_Info].
Global_Data_Info :: struct {
    version_stamp: u64;
    segment_info: [] Global_Data_Segment_Info;
}

// One segment of the global data: what kind it is (zeroed, writable, read-only, ...) and its bytes.
Global_Data_Segment_Info :: struct {
    segment_tag: enum u16 {
        BSS :: 0;
        DATA :: 1;
        RDATA :: 2;
        NO_RESET :: 3;
        USER :: 5;
    };
    data: [] u8;
}

// What a program knows about itself at run time: its type table and its global data. Get it from [get_runtime_info].
Runtime_Info :: struct {
    type_table: [] *Type_Info;
    global_data_info: *Global_Data_Info;
}

// Switches for how much type information a workspace keeps, set with [compiler_set_type_info_flags].
Type_Info_Flags :: enum u32 {
    NO_TYPE_INFO :: 0x1;
    PROCEDURES_ARE_VOID_POINTERS :: 0x2;
    NO_SIZE_COMPLAINT :: 0x4;
}

// Every setting of a build: what to produce (`output_type`), where to write it, the checks to compile in, the optimizer and the backend. Get a copy with [get_build_options], change it, and apply it with [set_build_options]. [set_optimization] fills in the fields that go with an optimization level.
//
// ```jai
// options: Build_Options;
// assert(options.output_type == .EXECUTABLE);
// options.output_type = .NO_OUTPUT;
// options.array_bounds_check = .OFF;
// ```
Build_Options :: struct {
    Output_Type :: enum u8 {
        NO_OUTPUT :: 0;
        EXECUTABLE :: 1;
        DYNAMIC_LIBRARY :: 2;
        STATIC_LIBRARY :: 3;
        OBJECT_FILE :: 4;
    };
    Os_Version :: struct {
        major: u8;
        minor: u8;
    };
    output_type := Output_Type.EXECUTABLE;
    using Commonly_Propagated: struct {
        write_added_strings := true;
        runtime_storageless_type_info := false;
        shorten_filenames_in_error_messages := false;
        use_visual_studio_message_format := false;
        use_natvis_compatible_types := false;
        minimum_os_version: Os_Version;
        lazy_foreign_function_lookups := false;
        disable_redzone := false;
        enable_bytecode_inliner := true;
        enable_bytecode_deduplication := true;
        stack_trace := true;
        use_ansi_color := true;
        interactive_bytecode_debugger := false;
        debug_for_expansions := false;
        enable_frame_pointers := true;
        runtime_support_definitions: enum u8 {
            AUTO;
            ENTRY_POINT_AND_INIT;
            ONLY_INIT;
            OMIT;
        } = .AUTO;
        backtrace_on_crash: enum u8 {
            OFF;
            ON;
        } = .ON;
        array_bounds_check: enum u8 {
            OFF;
            ON;
            ALWAYS;
        } = .ON;
        cast_bounds_check: enum u8 {
            OFF;
            NONFATAL;
            FATAL;
        } = .FATAL;
        null_pointer_check: enum u8 {
            OFF;
            ON;
        } = .ON;
        arithmetic_overflow_check: enum u8 {
            OFF;
            NONFATAL;
            FATAL;
        } = .OFF;
        dead_code_elimination: enum u8 {
            NONE;
            ALL;
            MODULES_ONLY;
        } = .MODULES_ONLY;
        max_bytecode_instructions_for_inlined_initializer: s32 = 8;
        context_size_max: s32 = 4096;
        prevent_compile_time_calls_from_runtime := false;
        info_flags: enum_flags u32 {
            POLYMORPH_MATCH :: 0x1;
            POLYMORPH_DEDUPLICATE :: 0x2;
        }
        text_output_flags: enum_flags u32 {
            OUTPUT_LINK_LINE :: 0x1;
            OUTPUT_TIMING_INFO :: 0x2;
        } = xx 3;
        os_target := OS;
        cpu_target := CPU;
        backend: enum u32 {
            X64;
            LLVM;
        } = .LLVM;
        machine_options: [MACHINE_OPTIONS_SIZE] u8;
        emit_debug_info: enum u32 {
            NONE :: 0;
            DWARF :: 1;
            CODEVIEW :: 2;
            DEFAULT :: 3;
        } = .DEFAULT;
        maximum_polymorph_depth := 100;
        maximum_array_count_before_compile_time_returns_are_not_reflected_in_ast := 5000;
        x64_options: X64_Options;
        llvm_options: Llvm_Options;
    }
    output_executable_name: string;
    output_path: string;
    intermediate_path: string;
    entry_point_name: string;
    compile_time_command_line: [] string;
    append_executable_filename_extension := true;
    use_custom_link_command := false;
    temporary_storage_size: s32 = 32768;
    import_path: [] string;
    additional_linker_arguments: [] string;
    user_data_u64: u64;
    user_data_string: string;
    user_data_pointer: *void;
    user_data_pointer_size: s32;
}

// The build options that can still change while a build is running. Apply them with [set_build_options_dc].
Build_Options_During_Compile :: struct {
    do_output := true;
    write_added_strings := true;
    append_executable_filename_extension := true;
    interactive_bytecode_debugger := false;
    append_linker_arguments: [] string;
    output_executable_name: string;
    output_path: string;
}

// How hard LLVM optimizes the program: `O0` to `O3` for speed, `OS` and `OZ` for size.
Llvm_Bitcode_Optimization_Setting :: enum u8 {
    UNSET :: 0;
    O0 :: 1;
    O1 :: 2;
    O2 :: 3;
    O3 :: 4;
    OS :: 5;
    OZ :: 6;
}

// How hard LLVM optimizes when it turns the program into machine code.
Llvm_Machine_Code_Optimization_Setting :: enum u8 {
    UNSET :: 0;
    NONE :: 1;
    LESS :: 2;
    DEFAULT :: 3;
    AGGRESSIVE :: 4;
}

// Settings for the LLVM backend: optimization levels, inlining, vectorization, and what intermediate output to keep.
Llvm_Options :: struct {
    bitcode_optimization_setting: Llvm_Bitcode_Optimization_Setting;
    machine_code_optimization_setting: Llvm_Machine_Code_Optimization_Setting;
    function_sections: bool = false;
    enable_tail_calls := false;
    enable_loop_unrolling := false;
    enable_slp_vectorization := false;
    enable_loop_vectorization := false;
    preserve_debug_info := true;
    merge_functions := false;
    disable_inlining := true;
    disable_mem2reg := false;
    enable_split_modules := true;
    output_bitcode_before_optimizations := false;
    output_llvm_ir_before_optimizations := false;
    output_bitcode := false;
    output_llvm_ir := false;
    target_system_triple: string;
    target_system_cpu: string;
    target_system_features: string;
    command_line: [] string;
}

// An optimization preset for [set_optimization], from `DEBUG` to `VERY_OPTIMIZED`, plus two that favor size.
Optimization_Type :: enum u8 {
    DEBUG :: 0;
    VERY_DEBUG :: 1;
    OPTIMIZED :: 2;
    VERY_OPTIMIZED :: 3;
    OPTIMIZED_SMALL :: 4;
    OPTIMIZED_VERY_SMALL :: 5;
}

// The compiler's version, filled in by [compiler_get_version_info].
Version_Info :: struct {
    major: s32;
    minor: s32;
    micro: s32;
}

// Settings for the built-in x64 backend.
X64_Options :: struct {
    use_dlls := false;
    enable_register_allocation := false;
    enable_unix_runtime_frame_information := true;
}

// Which kinds of typechecked code [compiler_begin_intercept] should leave out of its messages, and which performance reports to send.
Intercept_Flags :: enum_flags u32 {
    SKIP_EXPRESSIONS_WITHOUT_NOTES :: 0x1;
    SKIP_DECLARATIONS :: 0x2;
    SKIP_PROCEDURE_HEADERS :: 0x4;
    SKIP_PROCEDURE_BODIES :: 0x8;
    SKIP_STRUCTS :: 0x10;
    SKIP_OTHERS :: 0x20;
    SKIP_ALL :: SKIP_EXPRESSIONS_WITHOUT_NOTES | SKIP_DECLARATIONS | SKIP_PROCEDURE_HEADERS | SKIP_PROCEDURE_BODIES | SKIP_STRUCTS | SKIP_OTHERS;
    DO_PERFORMANCE_REPORT_POLYMORPHS :: 0x1000;
    DO_PERFORMANCE_REPORT_RUNS :: 0x2000;
}

// The start of every compiler message. `kind` says which `Message_*` type it really is; cast to that type to read the rest. `workspace` is the build it belongs to.
Message :: struct {
    kind: enum u8 {
        UNINITIALIZED :: 0;
        FILE;
        IMPORT;
        FAILED_IMPORT;
        PHASE;
        TYPECHECKED;
        COMPLETE;
        DEBUG_DUMP;
        ERROR;
        PERFORMANCE_REPORT;
    };
    workspace: Workspace;
}

// The last message of a build. `error_code` says whether it worked.
Message_Complete :: struct {
    #as using m: Message;
    error_code: enum u8 {
        NONE :: 0;
        COMPILATION_FAILED :: 1;
        COMPILER_SHUTDOWN :: 2;
    };
}

// The text of a `#debug_dump` request.
Message_Debug_Dump :: struct {
    #as using m: Message;
    dump_text: string;
}

// A module could not be found. Answer it with [provide_import], or the build fails.
Message_Failed_Import :: struct {
    #as using m: Message;
    status: enum u8 {
        UNINITIALIZED :: 0;
        BLOCKED :: 1;
        NOT_FOUND :: 2;
    };
    host_module_name: string;
    target_module_name: string;
    import_code: *Code_Directive_Import;
}

// A source file was loaded, with its full path and the import it belongs to.
Message_File :: struct {
    #as using m: Message;
    fully_pathed_filename: string;
    enclosing_import: *Message_Import;
    from_a_string: bool;
}

// A module was imported, with its name, path and kind.
Message_Import :: struct {
    #as using m: Message;
    module_type: enum u8 {
        UNINITIALIZED :: 0;
        PRELOAD :: 1;
        RUNTIME_SUPPORT :: 2;
        MAIN_PROGRAM :: 3;
        FILE :: 4;
    };
    module_name: string;
    fully_pathed_filename: string;
}

// Timing and counts about the build, sent when [Intercept_Flags] asks for them.
Message_Performance_Report :: struct {
    #as using m: Message;
    time_report: Time_Report;
    run_directive_report: Run_Directive_Report;
    polymorph_report: Polymorph_Report;
    bytecode_report: Bytecode_Report;
}

// The build reached a stage, such as all source being parsed or the executable being written.
Message_Phase :: struct {
    #as using m: Message;
    phase: enum u32 {
        ALL_SOURCE_CODE_PARSED :: 0;
        TYPECHECKED_ALL_WE_CAN :: 1;
        ALL_TARGET_CODE_BUILT :: 2;
        PRE_WRITE_EXECUTABLE :: 3;
        POST_WRITE_EXECUTABLE :: 4;
        READY_FOR_CUSTOM_LINK_COMMAND :: 5;
    };
    executable_name: string;
    executable_write_failed := false;
    linker_exit_code: s32;
    num_items_waiting_to_typecheck: s32;
    compiler_generated_object_files: [] string;
    support_object_files: [] string;
    system_libraries: [] string;
    user_libraries: [] string;
}

// A batch of code finished type checking: declarations, procedure headers and bodies, structs and everything else, each with the nodes inside it.
Message_Typechecked :: struct {
    #as using m: Message;
    declarations: [] Typechecked(Code_Declaration);
    procedure_headers: [] Typechecked(Code_Procedure_Header);
    procedure_bodies: [] Typechecked(Code_Procedure_Body);
    structs: [] Typechecked(Code_Struct);
    others: [] Typechecked(Code_Node);
    all: [] Typechecked(Code_Node);
}

// What the answer to a [Message_Failed_Import] is: a module name, a file, a folder or source text.
Provided_Import_Type :: enum u8 {
    SHORT_NAME :: 0;
    PATH_TO_FILE :: 1;
    PATH_TO_DIRECTORY :: 2;
    FULL_TEXT :: 3;
}

// How serious a [compiler_report] is. `ERROR` fails the build; the others only print.
Report :: enum u8 {
    ERROR :: 0;
    ERROR_CONTINUABLE :: 1;
    WARNING :: 2;
    INFO :: 3;
}

// Whether a workspace is still good, for [compiler_set_workspace_status].
Workspace_Status :: enum u8 {
    OK :: 0;
    FAILED :: 1;
}

// One argument in a call or return: its expression, and its name if it was passed by name.
Code_Argument :: struct {
    expression: *Code_Node;
    name: *Code_Ident;
}

// The details of an array literal: element type, alignment and members.
Code_Array_Literal_Info :: struct {
    element_type: *Code_Type_Instantiation;
    alignment: *Code_Node;
    array_members: [] *Code_Node;
    array_literal_flags: enum_flags u8 {
        WAS_UNTYPED :: 0x01;
    };
}

// An inline assembly block.
Code_Asm :: struct {
    #as using base: Code_Node;
    b1: *void;
    b2: [] *void;
    b3: [] u64;
}

// A two-operand expression such as `a + b`: the operator and the `left` and `right` nodes.
Code_Binary_Operator :: struct {
    Flags :: enum_flags u16 {
        SHIFT_MARKED_AS_LOGICAL :: 0x01;
        SHIFT_MARKED_AS_SMALL :: 0x02;
    }
    #as using base: Code_Node = .{ kind = .BINARY_OPERATOR };
    operator_type: Operator_Type;
    flags: Flags;
    left: *Code_Node;
    right: *Code_Node;
}

// A `{ ... }` block: its statements and the declarations it makes.
Code_Block :: struct {
    #as using base: Code_Node;
    parent: *Code_Block;
    block_type: enum s32 {
        UNINITIALIZED :: 0;
        IMPERATIVE :: 1;
        DATA_DECLARATIONS :: 2;
        ARGUMENTS :: 3;
        RETURNS :: 4;
        STRUCT_ARGUMENTS :: 5;
        CONSTANTS :: 6;
    };
    block_flags: enum u32 {
        NO_ARRAY_BOUNDS_CHECK :: 0x1;
        NO_ARITHMETIC_OVERFLOW_CHECK :: 0x2;
    };
    belongs_to_struct: *Code_Struct;
    members: [] *Code_Scope_Entry;
    statements: [] *Code_Node;
    owning_statement: *Code_Node;
}

// One `case` of an `if ==` switch.
Code_Case :: struct {
    #as using base: Code_Node;
    condition: *Code_Node;
    then_block: *Code_Block;
    owning_if: *Code_If;
    marked_as_fallthrough := false;
}

// A cast, explicit or inserted by the compiler, with the type it converts to.
Code_Cast :: struct {
    #as using base: Code_Node;
    target_type: *Code_Type_Instantiation;
    expression: *Code_Node;
    cast_flags: enum_flags u32 {
        IS_IMPLICIT :: 0x1;
        IS_POINTER_DEREFERENCE :: 0x2;
        IS_AUTO :: 0x4;
        NO_BOUNDS_CHECK :: 0x8;
        TRUNCATE :: 0x10;
        AS :: 0x20;
        ISA :: 0x40;
        FORCE :: 0x80;
        VERY_FORCE :: 0x100;
        HAS_FUNCTION_SYNTAX :: 0x800;
        HAS_POSTFIX_SYNTAX :: 0x1000;
    };
}

// One entry of a [Code_Comma_Separated_Arguments] list.
Code_Comma_Separated_Argument :: struct {
    node: *Code_Node;
    modifier: enum u8 {
        NONE :: 0;
        DECLARE :: 1;
        ASSIGN :: 2;
    };
}

// A comma-separated list on the left of `:=` or `=`, as in `a, b := f();`.
Code_Comma_Separated_Arguments :: struct {
    #as using base: Code_Node;
    arguments: [] Code_Comma_Separated_Argument;
}

// A declaration of several names at once, as in `a, b := f();`.
Code_Compound_Declaration :: struct {
    #as using entry: Code_Node;
    comma_separated_assignment: *Code_Comma_Separated_Arguments;
    declaration_properties: *Code_Declaration;
    alignment_expression: *Code_Node;
    notes: [] *Code_Note;
    operator_type: Operator_Type;
}

// The `context` value.
Code_Context :: struct {
    #as using base: Code_Node;
    kind = .CONTEXT;
}

// A declaration: a variable, constant, parameter or procedure name, with its type, value, notes and flags.
Code_Declaration :: struct {
    #as using entry: Code_Scope_Entry;
    type_inst: *Code_Type_Instantiation;
    expression: *Code_Node;
    flags: enum_flags u32 {
        IS_CONSTANT :: 0x1;
        IS_MARKED_AS_AS :: 0x2;
        IS_MARKED_AS_DISCARD :: 0x4;
        IS_ITERATOR :: 0x10;
        NO_RESET :: 0x40;
        IS_UNINITIALIZED :: 0x80;
        COMES_FROM_ASM :: 0x100;
        IS_IMPORTED :: 0x200;
        AUTO_VALUE_BAKE :: 0x4000;
        AUTO_VALUE_BAKE_IS_REQUIRED :: 0x8000;
        IS_OVERLAY :: 0x1_0000;
        PROGRAM_EXPORT :: 0x2_0000;
        ELSEWHERE :: 0x4_0000;
        SCOPE_FILE :: 0x8_0000;
        IS_GLOBAL :: 0x10_0000;
        HAS_SCOPE_MODIFIER :: 0x20_0000;
    };
    alignment_expression: *Code_Node;
    notes: [] *Code_Note;
    program_export_name: string;
}

// A `defer` statement and its block.
Code_Defer :: struct {
    #as using base: Code_Node;
    block: *Code_Block;
    is_backticked: bool;
}

// An `#add_context` directive.
Code_Directive_Add_Context :: struct {
    #as using base: Code_Node;
    expression: *Code_Node;
}

// A `#bake` or `$$` expression, which fixes some arguments of a procedure.
Code_Directive_Bake :: struct {
    #as using base: Code_Node;
    procedure_call: *Code_Procedure_Call;
    bake_type: enum u8 {
        CONSTANTS_BAKE :: 0;
        PARAMETER_VALUE_BAKE :: 1;
        DYNAMIC_SPECIALIZE :: 2;
    };
}

// A `#bytes` directive.
Code_Directive_Bytes :: struct {
    #as using base: Code_Node;
    expression: *Code_Node;
}

// A `#code` expression, whose body becomes a `Code` value.
Code_Directive_Code :: struct {
    #as using base: Code_Node;
    expression: *Code_Node;
    code_flags: enum u32 {
        IMPLICITLY_GENERATED :: 0x1;
        NULL :: 0x2;
        TYPED :: 0x8;
    };
}

// A `#context` directive.
Code_Directive_Context_Type :: struct {
    #as using base: Code_Node;
    kind = .DIRECTIVE_CONTEXT_TYPE;
}

// An `#exists` query.
Code_Directive_Exists :: struct {
    #as using base: Code_Node;
    query_expression: *Code_Node;
    sync_expression: *Code_Node;
}

// An `#import` directive: the module name and its parameters.
Code_Directive_Import :: struct {
    #as using base: Code_Node;
    name: string;
    flags: enum u32 {
        UNSHARED :: 0x1;
    };
    import_type: Provided_Import_Type;
    module_parameters_call: *Code_Procedure_Call;
    program_parameters_call: *Code_Procedure_Call;
}

// An `#insert` directive and the code it inserts.
Code_Directive_Insert :: struct {
    #as using base: Code_Node;
    expression: *Code_Node;
    scope_redirection: *Code_Node;
    break_replacement: *Code_Node;
    continue_replacement: *Code_Node;
    remove_replacement: *Code_Node;
    expansion: *Code_Node;
    is_internal := false;
}

// A `#library` directive naming a native library.
Code_Directive_Library :: struct {
    #as using base: Code_Node;
    name: string;
    library_flags: enum u32 {
        IS_SYSTEM_LIBRARY :: 0x1;
        DYNAMIC_LIBRARY_UNAVAILABLE :: 0x2;
        STATIC_LIBRARY_UNAVAILABLE :: 0x4;
        LINK_ALWAYS :: 0x8;
    };
}

// A `#load` directive: the file it names and its resolved path.
Code_Directive_Load :: struct {
    #as using base: Code_Node;
    short_name: string;
    fully_pathed_filename: string;
    loaded_string: string;
    load_flags: s32;
}

// A `#location` expression.
Code_Directive_Location :: struct {
    #as using base: Code_Node;
    expression: *Code_Node;
    is_caller_location := false;
}

// A `#modify` block on a procedure or struct.
Code_Directive_Modify :: struct {
    #as using base: Code_Node;
    block: *Code_Block;
}

// A `#module_parameters` directive.
Code_Directive_Module_Parameters :: struct {
    #as using base: Code_Node;
    module_parameters: *Code_Procedure_Header;
    program_parameters: *Code_Procedure_Header;
    common_code: *Code_Block;
}

// An `#overlay` directive.
Code_Directive_Overlay :: struct {
    #as using base: Code_Node;
    target_expression: *Code_Node;
    declaration_or_using: *Code_Node;
}

// A `#poke_name` directive.
Code_Directive_Poke_Name :: struct {
    #as using base: Code_Node;
    module_struct: *Code_Struct;
    name: string;
}

// A `#procedure_name` directive.
Code_Directive_Procedure_Name :: struct {
    #as using base: Code_Node;
    argument: *Code_Node;
}

// A `#run` directive and the procedure it runs.
Code_Directive_Run :: struct {
    #as using base: Code_Node;
    procedure: *Code_Procedure_Header;
    flags: enum u32 {
        ASSERTION :: 0x1;
        STALLABLE :: 0x2;
        SYNTACTICALLY_IMPLICIT :: 0x4;
        HAS_IMPLICIT_RETURN_TYPES :: 0x8;
    };
    assertion_string: string;
}

// A `#scope_export`, `#scope_file` or `#scope_module` directive.
Code_Directive_Scope :: struct {
    #as using base: Code_Node;
    scope_type: enum u32 {
        EXPORT :: 0;
        FILE :: 1;
        INTERNAL :: 2;
    };
}

// A `#through` directive.
Code_Directive_Through :: struct {
    #as using base: Code_Node;
    kind = .DIRECTIVE_THROUGH;
}

// A `#wildcard` placeholder.
Code_Directive_Wildcard :: struct {
    #as using base: Code_Node;
    index: s32 = -1;
}

// An enum definition: its members, notes and whether it is a flags or complete enum.
Code_Enum :: struct {
    #as using base: Code_Node;
    internal_type_inst: *Code_Type_Instantiation;
    internal_type: *Type_Info;
    external_type: *Type_Info_Enum;
    block: *Code_Block;
    notes: [] *Code_Note;
    marked_as_complete := false;
    marked_as_specified := false;
    is_flags := false;
}

// A `type_of`, `is_constant` or `#code_of` query about an expression.
Code_Expression_Query :: struct {
    #as using base: Code_Node;
    query_kind: enum u32 {
        TYPE_OF :: 0;
        IS_CONSTANT :: 1;
        CODE_OF :: 2;
    };
    expression_to_query: *Code_Node;
}

// Picks one value out of a multiple-value expression.
Code_Extract :: struct {
    #as using base: Code_Node;
    from: *Code_Node;
    index: s64;
}

// A `for` loop: what it iterates, its body and its `it` and `it_index` names.
Code_For :: struct {
    #as using base: Code_Node = .{ kind = .FOR };
    iteration_expression: *Code_Node;
    iteration_expression_right: *Code_Node;
    block: *Code_Block;
    ident_it: *Code_Ident;
    ident_it_index: *Code_Ident;
    ident_decl: *Code_Declaration;
    index_decl: *Code_Declaration;
    want_replacement_for_expansion: *Code_Node;
    want_pointer_expression: *Code_Node;
    want_reverse_expression: *Code_Node;
    macro_expansion_procedure_call: *Code_Procedure_Call;
    for_flags: For_Flags;
}

// A name, and the declaration it resolved to.
Code_Ident :: struct {
    #as using base: Code_Node = .{ kind = .IDENT };
    name: string;
    resolved_declaration: *Code_Declaration;
    flags: enum_flags u32 {
        DEFINES_POLYMORPH_VARIABLE :: 0x01;
        IS_RHS_OF_DOT_DEREFERENCE :: 0x02;
        RESOLVES_ONLY_SHALLOWLY :: 0x04;
        GENERATED_BY_OPERATOR_OVERLOAD :: 0x08;
        DO_NOT_RESOLVE_DUE_TO_PARSER :: 0x10;
        CAN_RESOLVE_TO_REG :: 0x20;
        HAS_SCOPE_MODIFIER :: 0x40;
    };
}

// An `if` or `ifx`, including a `#if` and an `if ==` switch.
Code_If :: struct {
    #as using base: Code_Node = .{ kind = .IF };
    condition: *Code_Node;
    then_block: *Code_Block;
    else_block: *Code_Block;
    if_flags: enum_flags u16 {
        IS_SWITCH_STATEMENT :: 0x1;
        IS_IFX :: 0x2;
        IS_STATIC :: 0x4;
        MARKED_AS_COMPLETE :: 0x8;
    };
    static_if_flags: enum_flags u16 {
        EVALUATED_AS_TRUE :: 0x1;
    };
    static_if_accepted_case: *Code_Case;
}

// A literal: a number, string, boolean, array, struct, pointer or type. `value_type` says which, and the matching union member holds the value.
Code_Literal :: struct {
    #as using base: Code_Node;
    kind = .LITERAL;

    value_type: enum s16 {
        UNINITIALIZED :: 0;
        NUMBER :: 1;
        STRING :: 2;
        BOOLEAN :: 3;
        ARRAY :: 6;
        STRUCT :: 7;
        POINTER :: 8;
        TYPE_INFO :: 9;
    }

    using values: union {
        _string: string;
        _float64: float64;
        _s64: s64;
        _u64: u64;
        struct_literal_info: *Code_Struct_Literal_Info;
        array_literal_info: *Code_Array_Literal_Info;
        pointer_literal_info: *Code_Pointer_Literal_Info;
        type_info_literal_defn: *Code_Type_Definition;
    };

    value_flags: enum_flags u32 {
        IS_A_NUMBER :: 0x1;
        HEX :: 0x2;
        FLOAT :: 0x4;
        BINARY :: 0x20;
        MINUS_SIGN :: 0x40;
        HERE_STRING :: 0x80;
        DEFAULTS_TO_FLOAT64 :: 0x100;
        REQUIRES_FLOAT64 :: 0x200;
    }
}

// A `break`, `continue` or `remove`.
Code_Loop_Control :: struct {
    #as using base: Code_Node;
    control_type: enum u8 {
        BREAK :: 0;
        CONTINUE :: 1;
        REMOVE :: 2;
    };
    target_ident: *Code_Ident;
}

// The list of extra arguments a call passes to a variadic procedure.
Code_Make_Varargs :: struct {
    #as using base: Code_Node;
    element_type: *Type_Info;
    expressions: [] *Code_Node;
    is_for_non_native_calling_convention: bool;
}

// The part every syntax node shares: its `kind`, where it is in the source, its type once checked and flags. Cast a node to the `Code_*` type that matches its `kind` to read the rest.
Code_Node :: struct {
    Location :: struct {
        enclosing_load: *Message_File;
        l0, c0, l1, c1: s32;
    }
    #as using location: Location;
    Kind :: enum u8 {
        UNINITIALIZED :: 0;
        BLOCK :: 1;
        LITERAL :: 2;
        IDENT :: 3;
        UNARY_OPERATOR :: 4;
        BINARY_OPERATOR :: 5;
        PROCEDURE_BODY :: 6;
        PROCEDURE_CALL :: 7;
        CONTEXT :: 8;
        WHILE :: 9;
        IF :: 10;
        LOOP_CONTROL :: 11;
        CASE :: 12;
        RETURN :: 14;
        FOR :: 15;
        TYPE_DEFINITION :: 16;
        TYPE_INSTANTIATION :: 17;
        ENUM :: 18;
        PROCEDURE_HEADER :: 19;
        STRUCT :: 20;
        COMMA_SEPARATED_ARGUMENTS :: 21;
        EXTRACT :: 22;
        DIRECTIVE_BYTES :: 23;
        MAKE_VARARGS :: 24;
        DECLARATION :: 25;
        CAST :: 26;
        DIRECTIVE_IMPORT :: 27;
        DIRECTIVE_THIS :: 28;
        DIRECTIVE_THROUGH :: 29;
        DIRECTIVE_LOAD :: 30;
        DIRECTIVE_RUN :: 31;
        DIRECTIVE_CODE :: 32;
        DIRECTIVE_POKE_NAME :: 33;
        ASM :: 34;
        DIRECTIVE_BAKE :: 35;
        DIRECTIVE_MODIFY :: 36;
        DIRECTIVE_LIBRARY :: 37;
        EXPRESSION_QUERY :: 38;
        PUSH_CONTEXT :: 39;
        NOTE :: 40;
        DIRECTIVE_PLACE :: 41;
        DIRECTIVE_SCOPE :: 42;
        TYPE_QUERY :: 43;
        DIRECTIVE_LOCATION :: 44;
        DIRECTIVE_MODULE_PARAMETERS :: 45;
        DIRECTIVE_ADD_CONTEXT :: 46;
        DIRECTIVE_COMPILE_TIME :: 47;
        COMPOUND_DECLARATION :: 48;
        DEFER :: 49;
        USING :: 50;
        PLACEHOLDER :: 51;
        DIRECTIVE_INSERT :: 52;
        DIRECTIVE_PROCEDURE_NAME :: 53;
        DIRECTIVE_WILDCARD :: 54;
        DIRECTIVE_EXISTS :: 55;
        DIRECTIVE_CONTEXT_TYPE :: 56;
        RESOLVED_OVERLOAD :: 57;
        DIRECTIVE_OVERLAY :: 58
    };
    kind := Kind.UNINITIALIZED;
    node_flags: enum u32 {
        EXPRESSION_IS_SPREAD :: 0x1;
        NO_ARRAY_BOUNDS_CHECK :: 0x2;
        ALLOWED_BY_CONTEXT :: 0x4;
        STATEMENT_IS_DEFERRED :: 0x8;
        IS_PARENTHESIZED :: 0x10;
        CREATED_BY_DESUGARING :: 0x100;
        EXPRESSION_IS_DEPRECATED :: 0x8000_0000
    };
    type: *Type_Info;
    serial: s64;
}

// A `@note` attached to a declaration, procedure or struct.
Code_Note :: struct {
    #as using base: Code_Node;
    text: string;
    note_flags: u32;
}

// A stand-in for code that is not there yet.
Code_Placeholder :: struct {
    #as using base: Code_Node;
    kind = .PLACEHOLDER;
}

// The details of a pointer constant: the symbol or literal it points into, and an offset.
Code_Pointer_Literal_Info :: struct {
    union {
        global_symbol: *Code_Declaration;
        string_or_array_literal: *Code_Literal;
    }
    data_pointer: *u8;
    pointer_literal_type: enum u8 {
        GLOBAL_SYMBOL :: 0;
        STRING_OR_ARRAY_LITERAL_DATA_POINTER :: 1;
        CONSTANT_VALUE :: 2;
    };
    offset_from_symbol: s64;
}

// The body of a procedure, with its header.
Code_Procedure_Body :: struct {
    #as using base: Code_Node = .{ kind = .PROCEDURE_BODY };
    block: *Code_Block;
    header: *Code_Procedure_Header;
    body_flags: enum u32 {
        ALREADY_MODIFIED :: 0x1;
    };
}

// A procedure call: the procedure, its arguments (as written and in parameter order) and the overload it resolved to.
Code_Procedure_Call :: struct {
    Flags :: enum_flags u32 {
        INLINE_YES :: 0x1;
        INLINE_NO :: 0x2;
        RETURNS_PROCEDURE_POINTER_ONLY :: 0x10;
        NO_DEBUG :: 0x100;
        IS_MODULE_PARAMETERS :: 0x200;
    }
    Context_Modification :: struct {
        modification_expressions: [] *Code_Node;
    }
    #as using base: Code_Node = .{ kind = .PROCEDURE_CALL };
    procedure_expression: *Code_Node;
    resolved_procedure_expression: *Code_Node;
    overloads: *[] *Code_Declaration;
    arguments_unsorted: [] Code_Argument;
    arguments_sorted: [] *Code_Node;
    num_return_values_received: s64;
    macro_expansion_block: *Code_Block;
    context_modification: *Context_Modification;
    flags: Flags;
}

// A procedure's declaration: name, parameters, return values, notes and flags such as `MACRO` or `POLYMORPHIC`.
Code_Procedure_Header :: struct {
    #as using base: Code_Node;
    constants_block: *Code_Block;
    arguments: [] *Code_Declaration;
    returns: [] *Code_Declaration;
    parameter_usings: [] *Code_Using;
    name: string;
    foreign_function_name: string;
    library_identifier: *Code_Ident;
    deprecation_string: string;
    polymorph_source_header: *Code_Procedure_Header;
    modify_directives: [] *Code_Directive_Modify;
    body_or_null: *Code_Procedure_Body;
    procedure_flags: enum_flags u32 {
        ELSEWHERE :: 0x1;
        COMPILE_TIME_ONLY :: 0x2;
        POLYMORPHIC :: 0x4;
        COMPILER_GENERATED :: 0x8;
        DEBUG_DUMP :: 0x10;
        C_CALL :: 0x20;
        TYPE_ONLY :: 0x40;
        INTRINSIC :: 0x80;
        DEPRECATED :: 0x100;
        SYNTACTICALLY_MARKED_AS_NO_CONTEXT :: 0x200;
        QUICK :: 0x400;
        QUICK_IN_BLOCK_FORM :: 0x800;
        CPP_METHOD :: 0x1000;
        NO_CALL :: 0x2000;
        MACRO :: 0x4000;
        NO_DEBUG :: 0x8000;
        ENTRY_POINT :: 0x1_0000;
        HAS_IMPLICIT_RETURN_VALUE :: 0x2_0000;
        SYMMETRIC :: 0x4_0000;
        CPP_RETURN_TYPE_IS_NON_POD :: 0x8_0000;
        ENTRY_POINT_HOOK :: 0x10_0000;
        SYNTACTICALLY_MARKED_AS_COMPILE_TIME :: 0x20_0000;
        SYNTACTICALLY_MARKED_AS_COMPILER :: 0x40_0000;
        SYNTACTICALLY_MARKED_AS_INLINE_YES :: 0x100_0000;
        SYNTACTICALLY_MARKED_AS_INLINE_NO :: 0x200_0000;
        INITIALIZER :: 0x400_0000;
    };
    notes: [] *Code_Note;
}

// A `push_context` statement and its block.
Code_Push_Context :: struct {
    #as using base: Code_Node;
    to_push: *Code_Node;
    block: *Code_Block;
    push_context_flags: enum u32 {
        IS_BACKTICKED :: 0x1;
        DEFER_POP :: 0x2;
    };
}

// An overloaded name after the compiler chose a procedure.
Code_Resolved_Overload :: struct {
    #as using base: Code_Node;
    result: *Code_Node;
    source_expression: *Code_Node;
}

// A `return` statement and its values.
Code_Return :: struct {
    Flags :: enum_flags u32 {
        IS_BACKTICKED :: 0x01;
        AUTO_INSERTED_FOR_QUICK_LAMBDA :: 0x02;
    }
    #as using base: Code_Node;
    arguments_unsorted: [] Code_Argument;
    arguments_sorted: [] *Code_Node;
    return_flags: Flags;
}

// Something named in a scope. [Code_Declaration] builds on it.
Code_Scope_Entry :: struct {
    #as using base: Code_Node;
    name: string;
    import_target: *Code_Declaration;
}

// A struct or union definition: its members, notes and the type it defines.
Code_Struct :: struct {
    #as using base: Code_Node;
    modify_directives: [] *Code_Directive_Modify;
    block: *Code_Block;
    arguments_block: *Code_Block;
    constants_block: *Code_Block;
    notes: [] *Code_Note;
    textual_flags: Struct_Textual_Flags;
    alignment: s32;
    defined_type: *Type_Info_Struct;
}

// The details of a struct literal: its type and its arguments.
Code_Struct_Literal_Info :: struct {
    type_expression: *Code_Type_Instantiation;
    arguments: [] *Code_Node;
}

// A type that is used as a value, with its `Type_Info`.
Code_Type_Definition :: struct {
    #as using base: Code_Node;
    info: *Type_Info;
}

// A type as written in source, such as `*[] int`, and the `Type_Info` it resolved to.
Code_Type_Instantiation :: struct {
    #as using base: Code_Node;
    result: *Type_Info;
    type_valued_expression: *Code_Node;
    must_implement: *Code_Node;
    pointer_to: *Code_Type_Instantiation;
    type_directive_target: *Code_Type_Instantiation;
    array_element_type: *Code_Type_Instantiation;
    array_dimension: *Code_Node;
    inst_flags: enum_flags u32 {
        VARARGS :: 0x1;
        RESIZABLE :: 0x2;
        TYPE_OF_PROCEDURE :: 0x4;
        TYPE_DIRECTIVE :: 0x8;
        TYPE_DIRECTIVE_DISTINCT :: 0x10;
        TYPE_DIRECTIVE_ISA :: 0x20;
        ARRAY_VIEW :: 0x40;
        INTERFACE :: 0x80;
    };
}

// A `size_of`, `type_info` or `initializer_of` expression.
Code_Type_Query :: struct {
    #as using base: Code_Node;
    query_kind: enum u32 {
        SIZE_OF :: 0;
        TYPE_INFO :: 1;
        INITIALIZER_OF :: 2;
    };
    type_to_query: *Code_Type_Instantiation;
}

// A one-operand expression such as `-x` or `!x`.
Code_Unary_Operator :: struct {
    #as using base: Code_Node;
    operator_type: Operator_Type;
    subexpression: *Code_Node;
}

// A `using` statement or parameter.
Code_Using :: struct {
    #as using base: Code_Node;
    expression: *Code_Node;
    filter_type: enum u8 {
        NONE :: 0;
        ONLY :: 1;
        EXCEPT :: 2;
        MAP :: 3;
    };
    filter_expression: *Code_Node;
    no_parameters := false;
}

// A `while` loop and its body.
Code_While :: struct {
    #as using base: Code_Node;
    condition: *Code_Node;
    block: *Code_Block;
}

// Why [get_root_type] succeeded or failed.
Get_Root_Type_Status :: enum u8 {
    UNINITIALIZED :: 0;
    SUCCESS :: 1;
    INPUT_IS_CODE_NULL :: 2;
    CORRUPTED :: 3;
    NOT_TYPED :: 4;
}

// An operator that has no single-character name. Single-character operators use their character, as in `cast(Operator_Type) #char "+"`.
Operator_Type :: enum s32 {
    IS_EQUAL :: 131;
    IS_NOT_EQUAL :: 132;
    LOGICAL_AND :: 133;
    LOGICAL_OR :: 134;
    LESS_OR_EQUAL :: 135;
    GREATER_OR_EQUAL :: 136;
    SHIFT_LEFT :: 137;
    SHIFT_RIGHT :: 138;
    ROTATE_LEFT :: 139;
    ROTATE_RIGHT :: 140;
    PLUS_ASSIGN :: 145;
    MINUS_ASSIGN :: 146;
    TIMES_ASSIGN :: 147;
    DIV_ASSIGN :: 148;
    MOD_ASSIGN :: 149;
    SHIFT_LEFT_ASSIGN :: 150;
    SHIFT_RIGHT_ASSIGN :: 151;
    ROTATE_LEFT_ASSIGN :: 152;
    ROTATE_RIGHT_ASSIGN :: 153;
    BITWISE_AND_ASSIGN :: 154;
    BITWISE_OR_ASSIGN :: 155;
    BITWISE_XOR_ASSIGN :: 156;
    LOGICAL_AND_ASSIGN :: 157;
    LOGICAL_OR_ASSIGN :: 158;
    POINTER_DEREFERENCE :: 168;
    POSTFIX_DEREFERENCE :: 169;
    ARRAY_SUBSCRIPT :: 500;
}

// How many calls and instructions the compile-time interpreter ran, and how many were inlined.
Bytecode_Report :: struct {
    num_calls_inlined: s64;
    num_calls_total: s64;
    num_instructions_inlined: s64;
    num_instructions_total: s64;
}

Compiler_Pin :: struct {
    name: string;
    pin: string;
}

// A plugin for a metaprogram. Fill in the procedures you care about and the compiler calls them: `init` once, `message` for every compiler message, `finish` at the end. `workspace` is the build it works on.
//
// ```jai
// plugin: Metaprogram_Plugin;
// plugin.message = (p: *Metaprogram_Plugin, message: *Message) {};
// assert(plugin.init == null);
// ```
Metaprogram_Plugin :: struct {
    workspace: Workspace;
    init: (p: *Metaprogram_Plugin, options: [] string) -> bool;
    before_intercept: (p: *Metaprogram_Plugin, flags: *Intercept_Flags);
    add_source: (p: *Metaprogram_Plugin);
    message: (p: *Metaprogram_Plugin, message: *Message);
    finish: (p: *Metaprogram_Plugin);
    shutdown: (p: *Metaprogram_Plugin);
    log_help: (p: *Metaprogram_Plugin);
    communication_proc: (p: *Metaprogram_Plugin, key: string, value: string) -> bool;
}

// Counts of the polymorphic procedures the compiler instantiated, and from where.
Polymorph_Report :: struct {
    Record :: struct {
        source: *Code_Procedure_Header;
        num_call_sites: s64;
        num_deduplications_by_constants: s64;
        num_deduplications_by_bytecode: s64;
        distinct_polymorphs: [] Distinct_Polymorph;
    }
    Distinct_Polymorph :: struct {
        polymorphed: *Code_Procedure_Header;
        call_site: *Code_Node;
        location: Source_Code_Location;
    }
    num_solves_invoked: s64;
    num_solves_used: s64;
    reported := false;
    records: [] Record;
}

// How long each `#run` took.
Run_Directive_Report :: struct {
    reported := false;

    Record :: struct {
        callee: *Code_Procedure_Header;
        num_stalls: s64;
        elapsed_time: float64;
    }

    records: [] Record;
}

// How long the compile-time interpreter spent running code, inlining and generating bytecode.
Time_Report :: struct {
    run_directives: float64;
    bytecode_inlining: float64;
    bytecode_generating: float64;
}

// A node that has finished type checking, with all the nodes inside it. Found in [Message_Typechecked].
Typechecked :: struct(T: Type) {
    expression: *T;
    subexpressions: [] *Code_Node;
}
