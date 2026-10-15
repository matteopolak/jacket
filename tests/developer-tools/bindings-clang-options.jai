#import "Basic";
B :: #import "Bindings_Generator";

native :: #foreign_library "../../target/developer_tools_options";
check_options :: (options: *B.CXIndexOptions) -> u32 #foreign native "developer_tools_check_options";

#assert size_of(B.CXIndexOptions) == 24;
#assert align_of(B.CXIndexOptions) == 8;
#assert offset_of(B.CXIndexOptions, "PreambleStoragePath") == 8;
#assert offset_of(B.CXIndexOptions, "InvocationEmissionPath") == 16;

main :: () {
    options: B.CXIndexOptions;

    assert(options.Size == 0 && options.__bitfield == 0 && options.PreambleStoragePath == null);
    options.Size = cast(u32) size_of(B.CXIndexOptions);

    for mask: 0..7 {
        options.__bitfield = 0xa5a0;
        B.set_ExcludeDeclarationsFromPCH(*options, (mask & 1) != 0);
        B.set_DisplayDiagnostics(*options, (mask & 2) != 0);
        B.set_StorePreamblesInMemory(*options, (mask & 4) != 0);
        assert(options.__bitfield == (0xa5a0 | cast(u16) mask));
        assert(B.get_ExcludeDeclarationsFromPCH(options) == ((mask & 1) != 0));
        assert(B.get_DisplayDiagnostics(options) == ((mask & 2) != 0));
        assert(B.get_StorePreamblesInMemory(options) == ((mask & 4) != 0));
        assert(check_options(*options) == cast(u32) mask);
        B.set_ExcludeDeclarationsFromPCH(*options, false);
        B.set_DisplayDiagnostics(*options, false);
        B.set_StorePreamblesInMemory(*options, false);
        assert(options.__bitfield == 0xa5a0);
    }

    options.__bitfield = 0;
    B.set_StorePreamblesInMemory(*options, true);
    index := B.clang_createIndexWithOptions(*options);

    assert(index != null);
    B.clang_disposeIndex(index);
    print("ok\n");
}
