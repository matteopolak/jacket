#import "Basic";
B :: #import "Bindings_Generator";

choose_enum :: (decl: *B.Declaration) -> string {
    if decl.name == "EXTRA_MEMBER" return "Existing";
    return "";
}

visit :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .ENUM {
        enumeration := cast(*B.Enum) decl;
        assert(enumeration.enumerates.count == 2);
        for value: enumeration.enumerates assert(value.parent == decl);
    }

    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-macro-merge.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_merge", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../target", #filepath));
    defer array_free(options.library_search_paths);
    options.convert_macro_value_to_enum_callback = choose_enum;
    options.visitor = visit;
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-macro-merge-generated.jai", #filepath), log_results = false));
    print("ok\n");
}
