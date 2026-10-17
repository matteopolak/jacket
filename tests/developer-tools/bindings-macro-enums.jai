#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

errors: s64;
mixed_selections: s64;
enum_selections: s64;

capture_error :: (_text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR {
        errors += 1;
    }
}

choose_mixed :: (decl: *B.Declaration) -> string {
    if decl.name == "ORIGINAL_INTEGER" || decl.name == "ORIGINAL_UNSIGNED" {
        mixed_selections += 1;
        return "MixedValues";
    }

    return "";
}

choose_enum :: (decl: *B.Declaration) -> string {
    assert(decl.kind == .MACRO_DEFINITION && decl.expression != null);

    if decl.name == "ORIGINAL_INTEGER" || decl.name == "ORIGINAL_NEGATIVE" || decl.name == "ORIGINAL_EXPRESSION" {
        enum_selections += 1;
        return "Options";
    }

    if decl.name == "ORIGINAL_UNSIGNED" {
        enum_selections += 1;
        return "WideOptions";
    }

    return "";
}

visit :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .FUNCTION && decl.name == "macro_native_mode" {
        assert(B.change_argument_type_to_enum(decl, "mode", "Options"));
    }

    if decl.kind == .FUNCTION && decl.name == "macro_native_unsigned" {
        assert(B.change_argument_type_to_enum(decl, "value", "WideOptions"));
    }

    return .RECURSE;
}

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-macro-values.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_macros", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../target", #filepath));
    defer array_free(options.library_search_paths);
    // A nonmatching prefix activates the callback without built-in grouping.
    array_add(*options.generate_enums_from_macros_with_prefixes, "UNUSED_BINDINGS_ENUM_");
    defer array_free(options.generate_enums_from_macros_with_prefixes);
    options.visitor = visit;
    options.convert_macro_value_to_enum_callback = choose_enum;
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-macro-enums-generated.jai", #filepath), log_results = false));
    assert(enum_selections == 4);
    mixed := tprint("%../../target/developer-tools-macro-mixed-generated.jai", #filepath);

    assert(write_entire_file(mixed, "existing bindings"));
    options.visitor = null;
    options.convert_macro_value_to_enum_callback = choose_mixed;

    push_context {
        context.logger = capture_error;
        assert(B.generate_bindings(options, mixed, log_results = false));
    }

    text, ok := read_entire_file(mixed);

    defer free(text);
    assert(ok && text != "existing bindings" && errors == 0 && mixed_selections == 2);
    assert(contains(text, "MixedValues :: enum") && contains(text, "0xffffffffffffffff"));
    print("ok\n");
}
