#import "Basic";
C :: #import "Codex";
Compiler :: #import "Compiler";
R :: #import "Relative_Pointers";

Tracking_Owner :: struct {
    backing: Allocator;
    frees: s64;
}

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    owner := cast(*Tracking_Owner) data;

    if mode == .FREE {
        assert(old_memory != null);
        owner.frees += 1;
    }

    return owner.backing.proc(mode, size, old_size, old_memory, owner.backing.data);
}

main :: () {
    header: C.Header_Record;

    assert(type_of(header.time_generated) == Apollo_Time);
    assert(type_of(header.OS) == Operating_System_Tag);
    assert(type_of(header.optimization_type) == Compiler.Optimization_Type);
    assert(type_of(header.project_name) == R.Relative_String(s16));
    assert(type_of(header.revision_control_type) == C.Revision_Control_Type);
    assert(cast(u64) C.Revision_Control_Type.PERFORCE == 5);
    module: C.Module_Record;
    file: C.File_Record;
    procedure: C.Procedure_Record;
    import_message: Compiler.Message_Import;
    procedure_header: Compiler.Code_Procedure_Header;

    assert(type_of(module.module_type) == type_of(import_message.module_type));
    assert(type_of(procedure.procedure_flags) == type_of(procedure_header.procedure_flags));
    assert(type_of(procedure.metrics) == C.Metrics);
    assert(size_of(type_of(header.reserved)) == 256);
    assert(cast(s64) *file.module_index - cast(s64) *file.fully_pathed_filename == size_of(R.Relative_String(s16)));
    assert(cast(s64) *procedure.metrics - cast(s64) *procedure.num_expressions == 4);
    borrowed: [16] u8;
    borrowed[0] = 93;
    help: C.Codex_Help;
    help.memory = borrowed;
    help.header = *header;
    modules: [1] *C.Module_Record;
    files: [1] *C.File_Record;
    procedures: [1] *C.Procedure_Record;

    modules[0] = *module;
    files[0] = *file;
    procedures[0] = *procedure;
    owner: Tracking_Owner;
    owner.backing = context.allocator;
    push_allocator(track, *owner);
    help.modules = array_copy(modules);
    help.files = array_copy(files);
    help.procedures = array_copy(procedures);
    C.deinit(*help);
    assert(help.memory.data == null && help.memory.count == 0 && help.header == null);
    assert(help.modules.data == null && help.files.data == null && help.procedures.data == null);
    assert(borrowed[0] == 93);
    C.deinit(*help);
    assert(owner.frees == 3);
    print("ok\n");
}
