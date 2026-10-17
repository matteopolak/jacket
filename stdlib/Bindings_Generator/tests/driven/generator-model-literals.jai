#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

#import "generator-model-template-layout-oracle";

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

errors: int;

log :: (_text: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

integer_value :: (literal: *B.Literal) -> u64 {
    if literal.literal_kind == .UNSIGNED_INTEGER return literal.uint_value;
    assert(literal.literal_kind == .INTEGER && literal.int_value >= 0);
    return cast(u64) literal.int_value;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-literals.hpp", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    args: [1] *u8;
    args[0] = to_c_string("-fblocks");
    defer free(args[0]);
    assert(B.clang_parseTranslationUnit2(index, filename, args.data, 1, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    capture: Capture;
    capture.saved = context;
    root := B.clang_getTranslationUnitCursor(unit);

    B.clang_visitChildren(root, visit, *capture);
    defer array_free(capture.children);
    assert(capture.children.count == 13);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);

    push_context {
        S.activate(*generator, null, *session);
        wchar := B.create_type_inner(B.clang_getCursorType(capture.children[7]), capture.children[7]);

        assert(wchar == generator.type_def_s32 && wchar.size == Native_WChar_Size && B.is_target_type_signed(wchar));

        if Native_LongDouble_Size == 8 {
            longdouble := B.create_type_inner(B.clang_getCursorType(capture.children[8]), capture.children[8]);
            assert(longdouble == generator.type_def_float64 && longdouble.size == Native_LongDouble_Size);
        }

        block := B.create_type_inner(B.clang_getTypedefDeclUnderlyingType(capture.children[9]), capture.children[9]);

        assert(block == generator.type_def_objc_block_pointer && block.type_of_struct != null);
        assert(block.type_of_struct.name == "Block" && block.type_of_struct.type == block && block.size == size_of(*void));
        assert(B.create_type_inner(B.clang_getCursorType(capture.children[10]), capture.children[10]).type_of_typedef.type == block);
        char16 := B.create_declaration(capture.children[0], root);
        char32 := B.create_declaration(capture.children[1], root);

        assert(char16 != null && char32 != null);
        assert(char16.type == generator.type_def_u16 && char16.type.size == Native_Char16_Size);
        assert(char32.type == generator.type_def_u32 && char32.type.size == Native_Char32_Size);
        assert(B.is_target_type_unsigned(char16.type) && B.is_target_type_unsigned(char32.type));
        assert(integer_value(cast(*B.Literal) char16.expression) == Native_Char16_Value);
        assert(integer_value(cast(*B.Literal) char32.expression) == Native_Char32_Value);
        utf8 := B.create_declaration(capture.children[2], root);

        assert(utf8 != null && utf8.expression.kind == .LITERAL);
        assert((cast(*B.Literal) utf8.expression).string_value == "Aé");
        empty := B.create_declaration(capture.children[5], root);

        assert(empty != null && empty.expression.kind == .LITERAL);
        assert((cast(*B.Literal) empty.expression).literal_kind == .STRING);
        assert((cast(*B.Literal) empty.expression).string_value.count == 0);
        zero := B.create_declaration(capture.children[6], root);

        assert(zero != null && integer_value(cast(*B.Literal) zero.expression) == 0);
        assert(!session.failed);
    }

    for i: 3..4 {
        rejection := S.open(*generator, B.TARGET_OS);

        push_context {
            S.activate(*generator, null, *rejection);
            context.logger = log;
            assert(B.create_declaration(capture.children[i], root) == null);
            assert(!rejection.failed && rejection.model_errors > 0 && errors == i - 2);
        }

        H.table_reset(*generator.declarations_by_cursor);
        S.close(*rejection);
    }

    payload_rejection := S.open(*generator, B.TARGET_OS);

    push_context {
        S.activate(*generator, null, *payload_rejection);
        context.logger = log;
        native := B.clang_getCanonicalType(B.clang_getCursorType(capture.children[12]));

        assert(native.kind == .Enum && B.clang_Type_getSizeOf(native) == 16);
        assert(B.create_literal(capture.children[12], root) == null);
        assert(!payload_rejection.failed && payload_rejection.model_errors > 0 && errors == 3);
    }

    S.close(*payload_rejection);
    assert(!session.failed);
    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
