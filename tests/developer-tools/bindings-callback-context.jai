#import "Basic";
B :: #import "Bindings_Generator";

Capture :: struct {
    saved_context: #Context;
    cursors: [..] B.CXCursor;
}

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved_context;
        array_add(*capture.cursors, cursor);
    }

    return .Continue;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%bindings-input.h", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    capture: Capture;
    capture.saved_context = context;
    defer array_free(capture.cursors);
    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *capture);
    assert(capture.cursors.count == 9);
    print("ok\n");
}
