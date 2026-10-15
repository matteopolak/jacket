#import "Basic";
#import "String";
B :: #import "Bindings_Generator";

#assert size_of(B.CXToken) == 24;
#assert offset_of(B.CXToken, "ptr_data") == 16;
#assert size_of(B.CXTokenKind) == 4;

#run {
    token_kind := cast(*Type_Info_Enum) type_info(B.CXTokenKind);
    #if OS == .WINDOWS assert(token_kind.internal_type.signed);
    else assert(!token_kind.internal_type.signed);
}

main :: () {
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string("original-token-fixture.h");

    defer free(filename);
    source :: "struct Original { int value; }; // retained\nint apply(struct Original *p) { return p->value + 7; }\n";
    contents := to_c_string(source);

    defer free(contents);
    unsaved: B.CXUnsavedFile = .{ Filename = filename, Contents = contents };
    unsaved.Length = cast(type_of(unsaved.Length)) source.count;
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, *unsaved, 1, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    root := B.clang_getTranslationUnitCursor(unit);
    tokens: *B.CXToken;
    count: u32;

    B.clang_tokenize(unit, B.clang_getCursorExtent(root), *tokens, *count);
    assert(tokens != null && count > 8);
    defer B.clang_disposeTokens(unit, tokens, count);
    annotations: [..] B.CXCursor;

    array_resize(*annotations, count);
    defer array_free(annotations);
    B.clang_annotateTokens(unit, tokens, count, annotations.data);
    first_spellings := string.["struct", "Original", "{", "int", "value", ";", "}", ";"];
    first_kinds := B.CXTokenKind.[.Keyword, .Identifier, .Punctuation, .Keyword, .Identifier, .Punctuation, .Punctuation, .Punctuation];
    saw_comment, saw_arrow, saw_literal: bool;
    previous_end: u32;

    for i: 0..cast(s64) count - 1 {
        spelling := B.clang_getTokenSpelling(unit, tokens[i]);

        defer B.clang_disposeString(spelling);
        text := to_string(B.clang_getCString(spelling));
        kind := B.clang_getTokenKind(tokens[i]);

        if i < first_spellings.count {
            assert(text == first_spellings[i] && kind == first_kinds[i]);
        }

        range := B.clang_getTokenExtent(unit, tokens[i]);
        first, last, location_offset: u32;
        first_file, last_file, location_file: B.CXFile;

        B.clang_getSpellingLocation(B.clang_getRangeStart(range), *first_file, null, null, *first);
        B.clang_getSpellingLocation(B.clang_getRangeEnd(range), *last_file, null, null, *last);
        B.clang_getSpellingLocation(B.clang_getTokenLocation(unit, tokens[i]), *location_file, null, null, *location_offset);
        assert(first_file != null && first_file == last_file && first_file == location_file);
        assert(location_offset == first && first >= previous_end && last > first && last <= source.count);
        assert(slice(source, first, last - first) == text);
        previous_end = last;

        if kind == .Comment {
            assert(text == "// retained");
            saw_comment = true;
        }

        if text == "->" {
            assert(kind == .Punctuation);
            saw_arrow = true;
        }

        if kind == .Literal {
            assert(text == "7");
            saw_literal = true;
        }

        if i == 1 {
            annotated := annotations[i];
            cursor := B.clang_getCursor(unit, B.clang_getTokenLocation(unit, tokens[i]));

            assert(B.clang_getCursorKind(annotated) == .StructDecl);
            assert(B.clang_equalCursors(annotated, cursor) != 0);
            assert(B.clang_equalCursors(cursor, B.clang_getNullCursor()) == 0);
        }
    }

    assert(saw_comment && saw_arrow && saw_literal);
    print("ok\n");
}
