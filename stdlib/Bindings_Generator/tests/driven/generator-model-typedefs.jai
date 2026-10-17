#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

Capture :: struct {
    saved: #Context;
    children: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        array_add(*capture.children, cur);
    }

    return .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-typedefs.h", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
    defer array_free(capture.children);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);
    defer H.table_reset(*generator.unwrapped_typedefs);
    options: B.Generate_Bindings_Options;

    array_add(*options.typedef_prefixes_to_unwrap, "Custom");
    defer array_free(options.typedef_prefixes_to_unwrap);
    tested := 0;

    push_context {
        S.activate(*generator, *options, *session);
        named := B.create_struct(capture.children[0]);

        assert(named.name == "OriginalNamed");

        for cur: capture.children {
            if cur.kind == .TypedefDecl {
                spelling := B.clang_getCursorSpelling(cur);

                defer B.clang_disposeString(spelling);
                name := to_string(B.clang_getCString(spelling));
                alias := B.create_typedef(cur);

                if name == "__kernel_OriginalAlias" || name == "CustomOriginalAlias" || name == "int32_t" {
                    assert(alias == null && B.create_typedef(cur) == null);
                    found, registered := H.table_find(*generator.unwrapped_typedefs, name);

                    assert(found && registered.name == name && registered.type != null);
                    if name == "int32_t" assert(registered.type == generator.type_def_s32);
                    else assert(registered.type.type_of_struct == named);

                    type := B.create_type(B.clang_getCursorType(cur), cur);

                    assert(type == registered.type);
                    tested += 1;
                } else if name == "OriginalAnonymous" {
                    assert(alias != null && (alias.decl_flags & .OMIT_FROM_OUTPUT) != 0);
                    record := alias.type.type_of_struct;

                    assert(record.name == name && record.output_name == name);
                    assert(B.create_struct(B.clang_getTypeDeclaration(B.clang_getTypedefDeclUnderlyingType(cur))) == record);
                    assert(record.declarations.count == 1 && record.name == name);
                    tested += 1;
                } else if name == "OriginalAnonymousEnum" {
                    assert(alias != null && (alias.decl_flags & .OMIT_FROM_OUTPUT) != 0);
                    enumeration := alias.type.type_of_enum;

                    assert(enumeration.name == name && enumeration.output_name == name);
                    assert(B.create_enum(B.clang_getTypeDeclaration(B.clang_getTypedefDeclUnderlyingType(cur))) == enumeration);
                    assert(enumeration.enumerates.count == 1 && enumeration.name == name);
                    tested += 1;
                } else {
                    assert(alias != null && (alias.decl_flags & .OMIT_FROM_OUTPUT) == 0);
                    assert(B.create_typedef(cur) == alias);
                    tested += 1;
                }
            } else if cur.kind == .VarDecl {
                variable := B.create_declaration(cur, B.clang_getTranslationUnitCursor(unit));
                assert(variable.type.pointer_to.type_of_struct == named);
                tested += 1;
            }
        }

        assert(tested == 8 && !session.failed);
    }

    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
