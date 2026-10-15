#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

Enum_Value :: struct {
    value: s64;
    found: bool;
    parent_is_enum: bool;
    start, end: u32;
}

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    if cursor.kind == .EnumConstantDecl {
        value := cast(*Enum_Value) data;
        value.value = B.clang_getEnumConstantDeclValue(cursor);
        value.found = true;
        value.parent_is_enum = B.clang_getCursorSemanticParent(cursor).kind == .EnumDecl;
        extent := B.clang_getCursorExtent(cursor);

        B.clang_getSpellingLocation(B.clang_getRangeStart(extent), null, null, null, *value.start);
        B.clang_getSpellingLocation(B.clang_getRangeEnd(extent), null, null, null, *value.end);
    }

    return .Recurse;
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string("original-unsaved-header.h");

    defer free(filename);
    contents := to_c_string("struct Broken { unknown_type field; };\n");

    defer free(contents);
    unsaved: B.CXUnsavedFile;
    unsaved.Filename = filename;
    unsaved.Contents = contents;
    unsaved.Length = cast(type_of(unsaved.Length)) to_string(contents).count;
    unit: B.CXTranslationUnit;
    status := B.clang_parseTranslationUnit2(index, filename, null, 0, *unsaved, 1, 0, *unit);

    assert(status == .Success && unit != null);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 1);
    diagnostic := B.clang_getDiagnostic(unit, 0);

    assert(diagnostic != null && B.clang_getDiagnosticSeverity(diagnostic) == .Error);
    defer B.clang_disposeDiagnostic(diagnostic);
    spelling := B.clang_getDiagnosticSpelling(diagnostic);

    defer B.clang_disposeString(spelling);
    assert(contains(to_string(B.clang_getCString(spelling)), "unknown_type"));
    formatted := B.clang_formatDiagnostic(diagnostic, B.clang_defaultDiagnosticDisplayOptions());

    defer B.clang_disposeString(formatted);
    assert(contains(to_string(B.clang_getCString(formatted)), "original-unsaved-header.h:1:17: error:"));
    location := B.clang_getDiagnosticLocation(diagnostic);

    assert(B.clang_Location_isFromMainFile(location) != 0);
    file: B.CXFile;
    line, column, offset: u32;

    B.clang_getSpellingLocation(location, *file, *line, *column, *offset);
    assert(file != null && line == 1 && column == 17 && offset == 16);
    name := B.clang_getFileName(file);

    defer B.clang_disposeString(name);
    assert(to_string(B.clang_getCString(name)) == "original-unsaved-header.h");
    fixup_contents := to_c_string("enum Signed { NEGATIVE=-3 }; struct Fixed { int field; }\n");

    defer free(fixup_contents);
    fixup := unsaved;
    fixup.Contents = fixup_contents;
    fixup.Length = cast(type_of(fixup.Length)) to_string(fixup_contents).count;
    second_unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, *fixup, 1, 0, *second_unit) == .Success);
    defer B.clang_disposeTranslationUnit(second_unit);
    assert(B.clang_getNumDiagnostics(second_unit) == 1);
    missing_semicolon := B.clang_getDiagnostic(second_unit, 0);

    defer B.clang_disposeDiagnostic(missing_semicolon);
    assert(B.clang_getDiagnosticNumFixIts(missing_semicolon) == 1);
    replacement_range: B.CXSourceRange;
    replacement := B.clang_getDiagnosticFixIt(missing_semicolon, 0, *replacement_range);

    defer B.clang_disposeString(replacement);
    assert(to_string(B.clang_getCString(replacement)) == ";");
    start, end: u32;

    B.clang_getSpellingLocation(B.clang_getRangeStart(replacement_range), null, null, null, *start);
    B.clang_getSpellingLocation(B.clang_getRangeEnd(replacement_range), null, null, null, *end);
    assert(start == end && start == fixup.Length - 1);
    value: Enum_Value;
    root := B.clang_getTranslationUnitCursor(second_unit);

    B.clang_visitChildren(root, visit, *value);
    assert(value.found && value.value == -3 && value.parent_is_enum);
    assert(value.end > value.start && value.end <= fixup.Length);
    assert(slice(to_string(fixup_contents), value.start, value.end - value.start) == "NEGATIVE=-3");
    print("ok\n");
}
