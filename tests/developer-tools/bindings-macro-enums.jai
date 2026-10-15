#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

replacement_types: [..] *B.CType;
errors: s64;
matched_error: bool;

capture_error :: (text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR {
        errors += 1;
        if contains(text, "matching native integer widths/signedness") matched_error = true;
    }
}

choose_incompatible :: (decl: *B.Declaration) -> string {
    if decl.name == "ORIGINAL_INTEGER" || decl.name == "ORIGINAL_UNSIGNED" return "Incompatible";
    return "";
}

choose_enum :: (decl: *B.Declaration) -> string {
    assert(decl.kind == .MACRO_DEFINITION && decl.expression != null);
    if decl.name == "ORIGINAL_INTEGER" || decl.name == "ORIGINAL_NEGATIVE" || decl.name == "ORIGINAL_EXPRESSION" return "Options";
    if decl.name == "ORIGINAL_UNSIGNED" return "WideOptions";
    return "";
}

visit :: (decl: *B.Declaration, _parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .FUNCTION && decl.name == "macro_native_mode" {
        assert(B.change_argument_type_to_enum(decl, "mode", "Options"));
        array_add(*replacement_types, decl.type.type_of_function.arguments[0].type);
    }

    if decl.kind == .FUNCTION && decl.name == "macro_native_unsigned" {
        assert(B.change_argument_type_to_enum(decl, "value", "WideOptions"));
        array_add(*replacement_types, decl.type.type_of_function.arguments[0].type);
    }

    return .RECURSE;
}

main :: () {
    defer {
        for type: replacement_types free(type);
        array_free(replacement_types);
    }

    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-macro-values.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_macros", identifier = "native" });
    defer array_free(options.libraries);
    options.visitor = visit;
    options.convert_macro_value_to_enum_callback = choose_enum;
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-macro-enums-generated.jai", #filepath), log_results = false));
    rejected := tprint("%../../target/developer-tools-macro-enums-rejected.jai", #filepath);

    assert(write_entire_file(rejected, "existing bindings"));
    options.visitor = null;
    options.convert_macro_value_to_enum_callback = choose_incompatible;

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bindings(options, rejected, log_results = false));
    }

    text, ok := read_entire_file(rejected);

    defer free(text);
    assert(ok && text == "existing bindings" && errors == 1 && matched_error);
    print("ok\n");
}
