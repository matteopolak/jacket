// Independently authored source schema for actual compiler message integration.
Intercept_Flags :: enum_flags u32 {
    SKIP_EXPRESSIONS_WITHOUT_NOTES :: 1;
    SKIP_DECLARATIONS :: 2;
    SKIP_PROCEDURE_HEADERS :: 4;
    SKIP_PROCEDURE_BODIES :: 8;
    SKIP_STRUCTS :: 16;
    SKIP_OTHERS :: 32;
    SKIP_ALL :: 63;
    DO_PERFORMANCE_REPORT_POLYMORPHS :: 0x1000;
    DO_PERFORMANCE_REPORT_RUNS :: 0x2000;
}
Message :: struct {
    kind: enum u8 {
        UNINITIALIZED; FILE; IMPORT; FAILED_IMPORT; PHASE;
        TYPECHECKED; COMPLETE; DEBUG_DUMP; ERROR; PERFORMANCE_REPORT;
    };
    workspace: s64;
}
Message_Phase :: struct {
    #as using m: Message;
    phase: enum u32 {
        ALL_SOURCE_CODE_PARSED; TYPECHECKED_ALL_WE_CAN; ALL_TARGET_CODE_BUILT;
        PRE_WRITE_EXECUTABLE; POST_WRITE_EXECUTABLE; READY_FOR_CUSTOM_LINK_COMMAND;
    };
    executable_name: string;
    executable_write_failed: bool;
    linker_exit_code: s32;
    num_items_waiting_to_typecheck: s32;
    compiler_generated_object_files: []string;
    support_object_files: []string;
    system_libraries: []string;
    user_libraries: []string;
}
Message_Complete :: struct {
    #as using m: Message;
    error_code: enum u8 { NONE; COMPILATION_FAILED; COMPILER_SHUTDOWN; };
}
compiler_begin_intercept :: (w: s64, flags: Intercept_Flags) #compiler;
compiler_end_intercept :: (w: s64) #compiler;
compiler_wait_for_message :: () -> *Message #compiler;
compiler_create_workspace :: (name: string) -> s64 #compiler;
add_build_string :: (data: string, w: s64) #compiler;
write_string :: (s: string, to_standard_error := false) #no_context #compiler;
