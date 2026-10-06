// TYPECHECKED, FILE and IMPORT messages built from compiler records, compiler_get_nodes,
// compiler_modify_procedure and add_build_string into a module (docs/metaprogramming/compiler-records.md).
// rules: compiler.15 compiler.16 records.3 records.4 records.6 records.7 records.8 records.9 records.18 records.27 ws.8
#import "Basic";
#import "Compiler";

TARGET :: #string END
#import "Basic";
Settings :: struct {
    width: int; @v1
    name:  string; @v2
}
VERSIONS :: Pair.[.{1, "width"}, .{2, "name"}];
Pair :: struct { version: int; member: string; }
Grid :: struct (T: Type, N: int) { cells: [N] T; }
Grid4 :: #bake_arguments Grid(N = 4);

counter: int;
bump :: () { counter += 10; }
work :: (cells: *Grid4(float32), scale := 2) -> int { counter += 1; return counter; } @Public
END

#run {
    w := compiler_create_workspace("target");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);

    inserted := compiler_get_nodes(#code bump());
    assert(inserted.kind == .PROCEDURE_CALL);

    main_import: *Message_Import;
    saw_settings, saw_versions, saw_work, modified := false;
    while true {
        message := compiler_wait_for_message();
        if message.kind == {
            case .IMPORT;
                import := cast(*Message_Import) message;
                if import.module_type == .MAIN_PROGRAM main_import = import;

            case .FILE;
                file := cast(*Message_File) message;
                assert(file.enclosing_import != null);

            case .TYPECHECKED;
                typechecked := cast(*Message_Typechecked) message;
                for typechecked.structs {
                    info := it.expression.defined_type;
                    if info.name == "Grid" assert(info.nontextual_flags & .POLYMORPHIC != 0);
                    if info.name != "Settings" continue;
                    saw_settings = true;
                    assert(info.members.count == 2 && info.members[0].notes[0] == "v1");
                }
                for typechecked.declarations {
                    decl := it.expression;
                    assert(decl.enclosing_load && decl.enclosing_load.enclosing_import == main_import);
                    if decl.name == "VERSIONS" {
                        saw_versions = true;
                        literal := cast(*Code_Literal) decl.expression;
                        assert(literal.value_type == .ARRAY);
                        entry := cast(*Code_Literal) literal.array_literal_info.array_members[1];
                        assert((cast(*Code_Literal) entry.struct_literal_info.arguments[0])._s64 == 2);
                        assert((cast(*Code_Literal) entry.struct_literal_info.arguments[1])._string == "name");
                    }
                    if decl.name == "work" {
                        saw_work = true;
                        assert(decl.notes.count == 1 && decl.notes[0].text == "Public");
                        assert(decl.type.type == .PROCEDURE);
                        header := cast(*Code_Procedure_Header) decl.expression;
                        assert(header.arguments.count == 2);
                        assert(header.arguments[1].type == type_info(int));
                        assert(header.arguments[1].expression.kind == .LITERAL);
                        // A baked struct's instance is the origin's instance, parameters included.
                        grid := cast(*Type_Info_Struct) (cast(*Type_Info_Pointer) header.arguments[0].type).pointer_to;
                        assert(grid.name == "Grid" && grid.specified_parameters.count == 2);
                        for param: grid.specified_parameters {
                            storage := grid.constant_storage.data + param.offset_into_constant_storage;
                            if param.name == "N" assert((cast(*int) storage).* == 4);
                            if param.name == "T" assert((cast(*Type) storage).* == float32);
                        }
                    }
                }
                for typechecked.procedure_bodies {
                    body := it.expression;
                    if body.header.name != "work" continue;
                    statements: [..] *Code_Node;
                    array_add(*statements, inserted);
                    for body.block.statements array_add(*statements, it);
                    body.block.statements = statements;
                    compiler_modify_procedure(w, body);
                    modified = true;
                }

            case .PHASE;
                phase := cast(*Message_Phase) message;
                if phase.phase == .TYPECHECKED_ALL_WE_CAN && main_import && modified {
                    // Runs the modified body: bump() then counter += 1.
                    add_build_string("#run assert(work(null) == 11);", w, main_import);
                    modified = false;
                }

            case .COMPLETE;
                assert((cast(*Message_Complete) message).error_code == .NONE);
                break;
        }
    }
    assert(saw_settings && saw_versions && saw_work);
    print("ok\n");
}
