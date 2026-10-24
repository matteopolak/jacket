#import "Basic";
B :: #import "Bindings_Generator";
H :: #import "Hash_Table";

S :: type_of(context._bindings_generator_session.*);

#import "generator-model-vtable-layout-oracle";

Capture :: struct {
    children: [..] B.CXCursor;
    saved: #Context;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.children, cur);
    }

    return .Continue;
}

errors: int;

log :: (_text: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-vtables.hpp", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
    defer array_free(capture.children);
    assert(capture.children.count == 11);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);
    defer H.table_reset(*generator.extra_infos);

    push_context {
        S.activate(*generator, null, *session);
        base := B.create_struct(capture.children[0], resolving_type_only = true);
        derived := B.create_struct(capture.children[1]);

        assert(base.type.size == Native_Virtual_Base_Size && derived.type.size == Native_Virtual_Derived_Size);
        assert(base.declarations.count == 0 && base.virtual_functions.count == 0);
        base_children: Capture;
        base_children.saved = context;
        B.clang_visitChildren(capture.children[0], visit, *base_children);
        defer array_free(base_children.children);
        omitted := B.create_function(base_children.children[2], capture.children[0]);

        assert(omitted != null);
        omitted.decl_flags |= .OMIT_FROM_OUTPUT;
        info, generated := B.find_or_create_extra_info(derived.type, cast(*B.Declaration) derived);

        assert(info != null && generated == derived);
        assert(info.vtable.count == 4 && info.vtable[0] == derived.virtual_functions[0] && info.vtable[1] == base.virtual_functions[1] && info.vtable[2] == derived.virtual_functions[1] && info.vtable[3] == derived.virtual_functions[2]);
        assert(info.vtable_struct != null && info.vtable_struct.declarations.count == 3);
        assert(info.vtable_struct.type.size == 4 * size_of(*void));
        // The table holds the slots the record adds after a member for the parent's table; an omitted slot is a pointer.
        assert(info.vtable_struct.declarations[0].type.type_of_struct == info.vtable_parent_info.vtable_struct && (info.vtable_struct.declarations[0].decl_flags & .QUALIFIER_USING) != 0);
        assert(info.vtable_parent_info.vtable_struct.declarations[1].type.pointer_to == generator.type_def_void);
        assert(info.vtable_struct.declarations[1].output_name == "hidden_1" && info.vtable_struct.declarations[1].bit_offset_in_parent == 16 * 8);
        assert(info.vtable_struct.declarations[1].type.type_of_function.arguments.count == 2);
        assert((info.vtable_struct.declarations[1].type.type_of_function.flags & .CPP_METHOD) != 0);
        assert(info.vtable_parent == base && info.vtable_parent_declaration == derived.parents[0]);
        assert(info.vtable_parent_info == H.table_find_pointer(*generator.extra_infos, base));
        assert(info.modified_declarations.count >= 6);
        assert(info.vtable_pointer_declaration.type.pointer_to.type_of_struct == info.vtable_struct);

        if info.modified_declarations[0] == info.vtable_pointer_declaration {
            // Itanium places a derived field in the base's tail padding: the base's fields are repeated.
            assert(info.modified_declarations[1].bit_offset_in_parent == Native_Virtual_Value_Offset * 8);
        } else {
            // MSVC keeps the base whole: it is embedded, with this record's table pointer over its own.
            embedded := info.modified_declarations[0];

            assert((embedded.decl_flags & .QUALIFIER_USING) != 0 && embedded.bit_offset_in_parent == 0);
            assert(info.modified_declarations[1] == info.vtable_pointer_declaration);
        }

        found_other := false;

        for field: info.modified_declarations if field.name == "other" {
            assert(field.bit_offset_in_parent == Native_Virtual_Other_Offset * 8);
            found_other = true;
        }

        assert(found_other);
        assert(info.struct_alignment == Native_Virtual_Alignment && derived.metadata.count == 1);
        again, again_generated := B.find_or_create_extra_info(derived.type, cast(*B.Declaration) derived);

        assert(again == info && again_generated == derived && derived.metadata.count == 1);

        // Hash-table growth must repair public cached parent-info links.
        for i: 0..39 {
            record_pointer := S.new_owned(*session, B.Struct);
            record_pointer.kind = .STRUCT;
            record_pointer.type = S.new_owned(*session, B.CType);
            record_pointer.type.type_of_struct = record_pointer;
            record_pointer.type.size = 1;
            record_pointer.type.alignment = 1;
            assert(B.find_or_create_extra_info(record_pointer.type, cast(*B.Declaration) record_pointer) != null);
        }

        info = H.table_find_pointer(*generator.extra_infos, derived);
        assert(info.vtable_parent_info == H.table_find_pointer(*generator.extra_infos, base));
        aligned := B.create_struct(capture.children[3]);
        aligned_info := B.find_or_create_extra_info(aligned.type, cast(*B.Declaration) aligned);

        assert(aligned_info != null && aligned.type.size == Native_Aligned_Size);
        assert(aligned_info.struct_alignment == Native_Aligned_Alignment);
        assert(aligned_info.modified_declarations.count == 5 && aligned_info.modified_declarations[1] == aligned_info.padding_declaration);
        assert(aligned_info.padding_declaration.type.size == Native_Aligned_Value_Offset - size_of(*void));
        assert(aligned_info.modified_declarations[2].bit_offset_in_parent == Native_Aligned_Value_Offset * 8);
        destructor := B.create_struct(capture.children[2]);
        destructor_info := B.find_or_create_extra_info(destructor.type, cast(*B.Declaration) destructor);

        assert(destructor_info && destructor_info.vtable.count == Native_Virtual_Destructor_Slots);
        for slot: destructor_info.vtable_struct.declarations assert(slot.type.pointer_to == generator.type_def_void);

        empty := B.create_struct(capture.children[4]);
        empty_info := B.find_or_create_extra_info(empty.type, cast(*B.Declaration) empty);

        assert(empty_info != null && empty_info.padding_declaration != null);
        assert(empty_info.padding_declaration.type.array_element_count == Native_Empty_Record_Size);
        forward := B.create_struct(capture.children[5]);
        forward_info, forward_generated := B.find_or_create_extra_info(forward.type, cast(*B.Declaration) forward);

        assert(forward_info == *B.struct_extra_info_unknown && forward_generated == null);
        assert(!H.table_contains(*generator.extra_infos, forward));

        push_context {
            context.logger = log;
            multiple := B.create_struct(capture.children[6]);

            assert(multiple && B.find_or_create_extra_info(multiple.type, cast(*B.Declaration) multiple) == null);
            assert(errors == 1 && !session.failed && session.model_errors > 0);
            assert(B.create_struct(capture.children[7]) == null);
            assert(errors == 2 && !session.failed);
            variadic := B.create_struct(capture.children[8]);

            assert(variadic && B.find_or_create_extra_info(variadic.type, cast(*B.Declaration) variadic) == null);
            assert(errors == 3 && !session.failed);
            packed := B.create_struct(capture.children[10]);

            assert(packed && B.find_or_create_extra_info(packed.type, cast(*B.Declaration) packed) == null);
            assert(errors == 4 && !session.failed);
        }

    }

    print("ok\n");
}
