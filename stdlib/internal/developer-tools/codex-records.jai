// Canonical public records; declaration order follows their dependency graph.
Revision_Control_Type :: enum u64 {
    UNKNOWN :: 0;
    SVN :: 1;
    GIT :: 2;
    KRAMPVS :: 3;
    MERCURIAL :: 4;
    PERFORCE :: 5;
}

Module_Record :: struct {
    module_type: type_of(Compiler.Message_Import.module_type) #align 1;
    module_name: Relative.Relative_String(s16) #align 1;
    fully_pathed_filename: Relative.Relative_String(s16) #align 1;
}

File_Record :: struct {
    fully_pathed_filename: Relative.Relative_String(s16) #align 1;
    module_index: s32 #align 1;
    from_a_string: bool #align 1;
}

Procedure_Record :: struct {
    file_index: s32 #align 1;
    l0: s32 #align 1;
    c0: s32 #align 1;
    l1: s32 #align 1;
    c1: s32 #align 1;
    procedure_flags: type_of(Compiler.Code_Procedure_Header.procedure_flags) #align 1;
    num_expressions: u32 #align 1;
    metrics: Metrics #align 1;
    name: Relative.Relative_String(s16) #align 1;
}

Header_Record :: struct {
    magic: u32;
    version: u32;
    time_generated: Basic.Apollo_Time #align 1;
    revision_control_type: Revision_Control_Type #align 1;
    revision_number: u64 #align 1;
    num_modules: s32 #align 1;
    num_files: s32 #align 1;
    num_procedures: s32 #align 1;
    reserved: [256] u8 #align 1;
    OS: Operating_System_Tag;
    optimization_type: Compiler.Optimization_Type;
    project_name: Relative.Relative_String(s16);
    executable_name: Relative.Relative_String(s16);
    url_or_generation_command: Relative.Relative_String(s16);
    comment: Relative.Relative_String(s16);
}

Codex_Help :: struct {
    memory: [] u8;
    header: *Header_Record;
    modules: [] *Module_Record;
    files: [] *File_Record;
    procedures: [] *Procedure_Record;
}

#scope_file
Basic :: #import "Basic";
Compiler :: #import "Compiler";
Relative :: #import "Relative_Pointers";
