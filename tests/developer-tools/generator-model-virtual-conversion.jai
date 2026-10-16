#import "Basic";
#import "String";
B :: #import "Bindings_Generator";
H :: #import "Hash_Table";

S :: type_of(context._bindings_generator_session.*);

diagnosed_missing_slot: bool;

capture_error :: (message: string, _data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 && contains(message, "unrepresented native virtual") diagnosed_missing_slot = true;
}

Capture :: struct {
    saved: #Context;
    record: B.CXCursor;
}

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved;
        if cursor.kind == .StructDecl capture.record = cursor;
    }

    return .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%generator-model-virtual-conversion.hpp", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    capture: Capture;
    capture.saved = context;
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
    assert(capture.record.kind == .StructDecl);
    generator: B.Generator;
    generator.translation_unit = unit;
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);
    defer H.table_reset(*generator.declarations_by_cursor);
    defer H.table_reset(*generator.extra_infos);

    push_context {
        S.activate(*generator, null, *session);
        record := B.create_struct(capture.record);

        assert(record != null && record.virtual_functions.count == 1);
        assert(record.virtual_functions[0].name == "following");
        context.logger = capture_error;
        info, generated := B.find_or_create_extra_info(record.type, cast(*B.Declaration) record);

        assert(info == null && generated == null);
        assert(!session.failed && session.model_errors > 0 && diagnosed_missing_slot);
        assert(!H.table_contains(*generator.extra_infos, record));
    }

    print("ok\n");
}
