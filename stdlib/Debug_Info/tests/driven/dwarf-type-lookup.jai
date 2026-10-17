#import "Basic";
D :: #import "Debug_Info";
BA :: #import "Bucket_Array";
#load "../../../../target/developer-tools-dwarf-type-oracle.jai";

errors: s64;

capture :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

#run {
    unit: D.Compilation_Unit;

    assert(type_of(unit.subprograms) == BA.Bucket_Array(D.Subprogram, 32));
    procedure: D.Subprogram;

    assert(type_of(procedure.all_blocks) == BA.Bucket_Array(D.Lexical_Block, 8));
    assert(type_of(unit.language) == D.Dwarf_Language);
    assert(cast(s64) D.Declaration.kind.PARAMETER == 0);
    assert(cast(s64) D.Declaration.kind.VARIABLE == 1);
}

main :: () {
    unit: D.Compilation_Unit;
    initial_type: D.Debug_Type;
    initial_declaration: D.Declaration;

    assert(initial_type.tag == .NULL && initial_type.offset == 0 && initial_type.size == 0);
    assert(cast(u16) unit.language == 0 && cast(s64) initial_declaration.kind == 0);

    assert(D.get_type(unit, 0) == null);
    array_add(*unit.types, D.Debug_Type.{ offset = 17, tag = .BASE_TYPE, name = "original integer", size = 4, base_type = .SIGNED });
    array_add(*unit.types, D.Debug_Type.{ offset = 91, tag = .POINTER_TYPE, size = 8 });
    defer array_free(unit.types);
    first := D.get_type(unit, 17);

    assert(first == *unit.types[0]);
    first.size = 16;
    assert(unit.types[0].size == 16);
    assert(D.get_type(unit, 91) == *unit.types[1]);
    assert(D.get_type(unit, 1) == null && D.get_type(unit, 0xffffffff) == null);
    array_add(*unit.types, unit.types[0]);

    push_context {
        context.logger = capture;
        assert(D.get_type(unit, 17) == null);
    }

    assert(errors == 1);
    check_native_type_lookups();
    print("ok\n");
}
