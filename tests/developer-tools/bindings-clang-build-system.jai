#import "Basic";
#import "File";
B :: #import "Bindings_Generator";

visit :: (cursor: B.CXCursor, _parent: B.CXCursor, data: B.CXClientData) -> B.CXChildVisitResult #c_call {
    if cursor.kind == .StructDecl && B.clang_isCursorDefinition(cursor) != 0 {
        count := cast(*s64) data;
        count.* += 1;
    }

    return .Continue;
}

main :: () {
    overlay := B.clang_VirtualFileOverlay_create(0);

    assert(overlay != null);
    defer B.clang_VirtualFileOverlay_dispose(overlay);
    relative := to_c_string("relative.h");

    defer free(relative);
    real_path := to_c_string(tprint("%bindings-input.h", #filepath));

    defer free(real_path);
    virtual_path := to_c_string(tprint("%virtual/Original.h", #filepath));

    defer free(virtual_path);
    assert(B.clang_VirtualFileOverlay_addFileMapping(overlay, relative, real_path) != .Success);
    assert(B.clang_VirtualFileOverlay_addFileMapping(overlay, virtual_path, real_path) == .Success);
    assert(B.clang_VirtualFileOverlay_setCaseSensitivity(overlay, 0) == .Success);
    buffer: *u8;
    size: u32;

    assert(B.clang_VirtualFileOverlay_writeToBuffer(overlay, 0, *buffer, *size) == .Success);
    assert(buffer != null && size > 0);
    defer B.clang_free(buffer);
    overlay_file := tprint("%../../target/developer-tools-overlay.yaml", #filepath);

    assert(write_entire_file(overlay_file, string.{ cast(s64) size, buffer }));
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    upper_path := to_c_string(tprint("%virtual/ORIGINAL.H", #filepath));

    defer free(upper_path);
    args := string.["-x", "c", "-ivfsoverlay", overlay_file];
    native_args: [..] *u8;

    defer {
        for argument: native_args free(argument);
        array_free(native_args);
    }

    for argument: args array_add(*native_args, to_c_string(argument));

    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, upper_path, native_args.data, cast(s32) native_args.count, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    assert(B.clang_getNumDiagnostics(unit) == 0);
    records: s64;

    B.clang_visitChildren(B.clang_getTranslationUnitCursor(unit), visit, *records);
    assert(records == 1);

    descriptor := B.clang_ModuleMapDescriptor_create(0);

    assert(descriptor != null);
    defer B.clang_ModuleMapDescriptor_dispose(descriptor);
    name := to_c_string("OriginalNative");

    defer free(name);
    header := to_c_string("OriginalNative.h");

    defer free(header);
    assert(B.clang_ModuleMapDescriptor_setFrameworkModuleName(descriptor, name) == .Success);
    assert(B.clang_ModuleMapDescriptor_setUmbrellaHeader(descriptor, header) == .Success);
    module_buffer: *u8;
    module_size: u32;

    assert(B.clang_ModuleMapDescriptor_writeToBuffer(descriptor, 0, *module_buffer, *module_size) == .Success);
    assert(module_buffer != null && module_size > 0);
    defer B.clang_free(module_buffer);
    assert(write_entire_file(tprint("%../../target/developer-tools-frameworks/OriginalNative.framework/Modules/module.modulemap", #filepath),
                            string.{ cast(s64) module_size, module_buffer }));
    print("ok\n");
}
