#import "Basic";
B :: #import "Bindings_Generator";

errors: s64;
inspected: bool;

inspect :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .FUNCTION && decl.name == "inspect_flags" {
        type := decl.type.type_of_function;

        assert(type != null && B.is_void_type(type.return_type));
        assert(type.arguments.count == 1 && B.is_bool_type(type.arguments[0].type));
        returned := type.return_type.*;
        returned.flags |= .CONST | .VOLATILE;
        assert(B.is_same_type(type.return_type, *returned));
        parameter := type.arguments[0].type.*;
        parameter.flags |= .CONST;
        assert(B.is_same_type(type.arguments[0].type, *parameter));
        inspected = true;
    }

    return .RECURSE;
}

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    empty, boolean, nothing: B.CType;
    boolean.flags = .BOOL | .CONST;
    nothing.flags = .VOID;
    assert(!B.is_bool_type(*empty) && !B.is_void_type(*empty));
    assert(B.is_bool_type(*boolean) && !B.is_void_type(*boolean));
    assert(B.is_void_type(*nothing) && !B.is_bool_type(*nothing));
    assert(!B.is_bool_type(null) && !B.is_void_type(null));
    alias: B.Typedef;
    alias.kind = .TYPEDEF;
    alias.name = "Original_Bool";
    alias.type = *boolean;
    type: B.CType;
    type.type_of_typedef = *alias;
    assert(B.is_bool_type(*type));
    alias.type = *nothing;
    assert(B.is_void_type(*type));
    pointer := B.create_pointer_to(*boolean);

    defer free(pointer);
    assert(!B.is_bool_type(pointer) && !B.is_void_type(pointer));
    alias.type = *type;

    push_context {
        context.logger = capture_error;
        assert(!B.is_bool_type(*type));
    }

    assert(errors == 1);
    index := B.clang_createIndex(0, 0);

    assert(index != null);
    defer B.clang_disposeIndex(index);
    filename := to_c_string(tprint("%bindings-input.h", #filepath));

    defer free(filename);
    unit: B.CXTranslationUnit;

    assert(B.clang_parseTranslationUnit2(index, filename, null, 0, null, 0, 0, *unit) == .Success);
    defer B.clang_disposeTranslationUnit(unit);
    file := B.clang_getFile(unit, filename);

    assert(file != null);
    location := B.Location.{ file = file, line = 3, column = 7 };
    same := location;

    assert(B.is_same_location(location, same));
    same.line += 1;
    assert(!B.is_same_location(location, same));
    same = location;
    same.column += 1;
    assert(!B.is_same_location(location, same));
    same = location;
    same.file = null;
    assert(!B.is_same_location(location, same));
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-model-properties.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_properties", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.visitor = inspect;
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-properties-generated.jai", #filepath), log_results = false));
    assert(inspected);
    print("ok\n");
}
