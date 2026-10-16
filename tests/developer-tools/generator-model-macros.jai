#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

errors: int;

log :: (_text: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

Capture :: struct {
    saved: #Context;
    source_file: B.CXFile;
    children: [..] B.CXCursor;
}

visit :: (cur: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;

        if cur.kind == .MacroDefinition {
            location: B.Location;
            B.fill_location(*location, cur);
            if location.file == capture.source_file array_add(*capture.children, cur);
        }
    }

    return .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-macros.h", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 1, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    capture: Capture;
    capture.saved = context;
    capture.source_file = B.clang_getFile(unit, filename);
    root := B.clang_getTranslationUnitCursor(unit);

    B.clang_visitChildren(root, visit, *capture);
    defer array_free(capture.children);
    assert(capture.children.count == 4);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);
    pending_value: *B.Declaration;

    push_context {
        context.generator = *generator;
        context._bindings_generator_session = *session;
        assert(!B.create_macro_definition(capture.children[0]));
        value := B.create_macro_definition(capture.children[1]);

        pending_value = value;

        assert(value != null && value.name == "ORIGINAL_MODEL_VALUE" && value.kind == .MACRO_DEFINITION);
        assert(value.expression.kind == .LITERAL && (cast(*B.Literal) value.expression).literal_kind == .MACRO);
        literal := cast(*B.Literal) value.expression;

        assert(literal.macro_value.tokens.count == 5);
        assert(B.is_token(literal.macro_value.tokens[0], "("));
        assert(B.find_closing_paren_token_index(literal.macro_value.tokens, 1) == 4);
        assert(B.create_macro_definition(capture.children[1]) == value);
        assert(!B.create_macro_definition(capture.children[2]));
        assert(!B.create_macro_definition(capture.children[3]));
        assert(generator.tokens_to_free.count == 1 && !session.failed);
    }

    pending_generator: B.Generator;
    pending_generator.translation_unit = unit;
    pending_generator.clang_index = index;
    scalar: B.CType;
    scalar.size = 4;
    scalar.alignment = 4;
    scalar.number_flags = .SIGNED | ._32BIT;
    pending_generator.type_def_s32 = *scalar;
    pending_value.type = *scalar;
    array_add(*pending_generator.global_scope.members, pending_value);
    defer array_free(pending_generator.global_scope.members);
    validation := S.open(*pending_generator, B.TARGET_OS);
    input: type_of(validation.parse_input.*);
    input.filename = to_string(filename);
    validation.parse_input = *input;

    push_context {
        S.activate(*pending_generator, null, *validation);
        context.logger = log;
        before := errors;

        pending_value.macro_value_result = .DISCARD;
        assert(!B.find_and_check_macro_identifier(pending_value.name, ignore_omit_flag = true));
        pending_value.macro_value_result = .KEEP;
        pending_value.decl_flags |= .OMIT_FROM_OUTPUT;
        assert(!B.find_and_check_macro_identifier(pending_value.name));
        assert(B.find_and_check_macro_identifier(pending_value.name, ignore_omit_flag = true));
        pending_value.decl_flags &= ~B.Declaration_Flags.OMIT_FROM_OUTPUT;
        pending_value.macro_value_result = .NEEDS_CHECKING;
        found, _name, _type, _enum := B.find_and_check_macro_identifier(pending_value.name);

        #if #exists(B.check_macro_value) {
            assert(pending_value.macro_value_result != .NEEDS_CHECKING);
            assert(found == (pending_value.macro_value_result == .KEEP));
        } else {
            assert(!found && !validation.failed && validation.model_errors > 0 && errors == before + 1);
            assert(pending_value.macro_value_result == .NEEDS_CHECKING);
        }
    }

    S.close(*validation);
    for info: generator.tokens_to_free B.clang_disposeTokens(unit, info.token_pointer, info.num_tokens);

    missing: B.Generator;
    rejected := S.open(*missing, B.TARGET_OS);

    push_context {
        S.activate(*missing, null, *rejected);
        context.logger = log;
        before := errors;

        assert(B.create_macro_definition(capture.children[1]) == null);
        assert(rejected.failed && errors == before + 1 && missing.tokens_to_free.count == 0);
    }

    S.close(*rejected);

    print("ok\n");
}

#scope_file
H :: #import "Hash_Table";
