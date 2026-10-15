#import "Basic";
B :: #import "Bindings_Generator";
#if OS != .WINDOWS P :: #import "POSIX";

#run {
    procedure := cast(*Type_Info_Procedure) type_info(type_of(B.clang_getFileTime));
    assert(procedure.return_types.count == 1);
    #if OS == .WINDOWS assert(procedure.return_types[0] == type_info(s64));
    else assert(procedure.return_types[0] == type_info(P.time_t));
}

main :: () {
    expected := s64.[0x100000123, -12345];
    suffixes := string.["future", "past"];
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);

    for suffix, number: suffixes {
        path := to_c_string(tprint("%../../target/developer-tools-time-%.h", #filepath, suffix));

        defer free(path);
        unit: B.CXTranslationUnit;

        assert(B.clang_parseTranslationUnit2(index, path, null, 0, null, 0, 0, *unit) == .Success);
        defer B.clang_disposeTranslationUnit(unit);
        assert(B.clang_getNumDiagnostics(unit) == 0);
        file := B.clang_getFile(unit, path);

        assert(file != null);
        timestamp := B.clang_getFileTime(file);

        assert(timestamp == expected[number]);
        assert(size_of(type_of(timestamp)) == 8);
    }

    print("ok\n");
}
