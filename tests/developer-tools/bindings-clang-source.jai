#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

Read_Location :: #type (location: B.CXSourceLocation, file: *B.CXFile, line: *u32, column: *u32, offset: *u32) -> void #c_call;

main :: () {
    null_location := B.clang_getNullLocation();

    assert(B.clang_equalLocations(null_location, B.clang_getNullLocation()) != 0);
    null_range := B.clang_getNullRange();

    assert(B.clang_Range_isNull(null_range) != 0);
    assert(B.clang_equalRanges(null_range, B.clang_getNullRange()) != 0);
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    path := tprint("%bindings-source-locations.h", #filepath);
    filename := to_c_string(path);

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    file := B.clang_getFile(unit, filename);

    assert(file != null && B.clang_File_isEqual(file, B.clang_getFile(unit, filename)) != 0);
    identity, repeated_identity: B.CXFileUniqueID;

    assert(B.clang_getFileUniqueID(file, *identity) == 0);
    assert(B.clang_getFileUniqueID(B.clang_getFile(unit, filename), *repeated_identity) == 0);
    for i: 0..2 assert(identity.data[i] == repeated_identity.data[i]);

    real_path := B.clang_File_tryGetRealPathName(file);

    defer B.clang_disposeString(real_path);
    assert(to_string(B.clang_getCString(real_path)) == path);
    bytes, read_ok := read_entire_file(path);

    assert(read_ok);
    defer free(bytes);
    native_size: u64;
    native_bytes := B.clang_getFileContents(unit, file, *native_size);

    assert(native_bytes != null && native_size == cast(u64) bytes.count);
    assert(string.{ cast(s64) native_size, native_bytes } == bytes);
    physical := B.clang_getLocation(unit, file, 3, 5);
    readers := Read_Location.[B.clang_getSpellingLocation, B.clang_getExpansionLocation,
                             B.clang_getInstantiationLocation, B.clang_getFileLocation];
    expected_offset: u32;
    found := false;
    name :: "original_source_function";

    for position: 0..bytes.count - name.count {
        if slice(bytes, position, name.count) == name {
            assert(!found);
            expected_offset = cast(u32) position;
            found = true;
        }
    }

    assert(found);

    for reader: readers {
        result_file: B.CXFile;
        line, column, offset: u32;
        reader(physical, *result_file, *line, *column, *offset);
        assert(B.clang_File_isEqual(result_file, file) != 0 && line == 3 && column == 5 && offset == expected_offset);
    }

    by_offset := B.clang_getLocationForOffset(unit, file, expected_offset);

    assert(B.clang_equalLocations(physical, by_offset) != 0);
    presumed_file: B.CXString;
    presumed_line, presumed_column: u32;

    B.clang_getPresumedLocation(physical, *presumed_file, *presumed_line, *presumed_column);
    defer B.clang_disposeString(presumed_file);
    assert(to_string(B.clang_getCString(presumed_file)) == "original-presumed.c");
    assert(presumed_line == 100 && presumed_column == 5);
    first := B.clang_getLocation(unit, file, 3, 1);

    assert(B.clang_isBeforeInTranslationUnit(first, physical) != 0);
    assert(B.clang_isBeforeInTranslationUnit(physical, first) == 0);
    assert(B.clang_isBeforeInTranslationUnit(physical, physical) == 0);
    range := B.clang_getRange(first, physical);

    assert(B.clang_Range_isNull(range) == 0 && B.clang_equalRanges(range, null_range) == 0);
    assert(B.clang_equalLocations(B.clang_getRangeStart(range), first) != 0);
    assert(B.clang_equalLocations(B.clang_getRangeEnd(range), physical) != 0);
    assert(B.clang_equalRanges(range, B.clang_getRange(first, physical)) != 0);
    cursor := B.clang_getCursor(unit, physical);

    assert(B.clang_getCursorKind(cursor) == .FunctionDecl && B.clang_isDeclaration(cursor.kind) != 0);
    assert(B.clang_isReference(cursor.kind) == 0 && B.clang_isInvalid(cursor.kind) == 0);
    assert(B.clang_isInvalidDeclaration(cursor) == 0);
    assert(B.clang_isInvalid(B.clang_getNullCursor().kind) != 0);
    set := B.clang_createCXCursorSet();

    assert(set != null);
    defer B.clang_disposeCXCursorSet(set);
    assert(B.clang_CXCursorSet_contains(set, cursor) == 0);
    assert(B.clang_CXCursorSet_insert(set, cursor) != 0);
    assert(B.clang_CXCursorSet_contains(set, cursor) != 0);
    assert(B.clang_CXCursorSet_insert(set, cursor) == 0);
    print("ok\n");
}
