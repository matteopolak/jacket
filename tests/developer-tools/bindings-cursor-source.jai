// Reviewed raw_tokens_string_from_cursor(CXCursor) and official libclang token ABI.
#import "Basic";
B :: #import "Bindings_Generator";

source := "int reconstructed(int value) { /* intentionally discarded */ return (value << 2) + 1; } const char *reconstructed_text() { return \"a\\\"b\"; }";

Capture :: struct {
    saved_context: #Context;
    children: [..] B.CXCursor;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    file := B.CXUnsavedFile.{ Filename = "independent-cursor.cpp", Contents = to_c_string(source), Length = cast(u64) source.count };

    defer free(file.Contents);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, file.Filename, null, 0, *file, 1, 0, *unit) == .Success);
    defer if unit B.clang_disposeTranslationUnit(unit);
    capture := Capture.{ saved_context = context };

    defer array_free(capture.children);
    assert(B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), collect, *capture) == 0);
    assert(capture.children.count == 2);
    text := B.raw_tokens_string_from_cursor(capture.children[0]);

    defer free(text);
    assert(text == "intreconstructed(intvalue){return(value<<2)+1;}");
    text2 := B.raw_tokens_string_from_cursor(capture.children[1]);

    defer free(text2);
    B.clang_disposeTranslationUnit(unit);
    unit = null;
    // These are token spellings, deliberately not compilable reconstructed C++.
    assert(text2 == "constchar*reconstructed_text(){return\"a\\\"b\";}");
    print("ok\n");
}

collect :: (cursor: B.CXCursor, parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    capture := cast(*Capture) data;

    push_context {
        context = capture.saved_context;
        array_add(*capture.children, cursor);
    }

    return .Continue;
}
