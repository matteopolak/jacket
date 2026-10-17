// Real public C++ workflow with explicitly flattened callback-provided names.
#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

namespaces: int;
records: int;
functions: int;
alpha_record: *B.Struct;
beta_record: *B.Struct;
flatten := true;
unique_names := true;
errors: int;
cycle_error: bool;
make_cycle: bool;
cycle_children: [..] *B.Declaration;

customize :: (decl: *B.Declaration, parent: *B.Declaration) -> B.Declaration_Visit_Result {
    if decl.kind == .NAMESPACE && decl.name {
        namespaces += 1;

        if flatten {
            namespace := cast(*B.Namespace) decl;
            namespace.flags |= .FLATTEN;
        }

        if make_cycle && decl.name == "alpha" {
            array_add(*cycle_children, decl);
            (cast(*B.Namespace) decl).declarations = cycle_children;
        }
    }

    if decl.kind == .STRUCT {
        records += 1;
        assert(parent.kind == .NAMESPACE);

        if parent.name == "alpha" {
            alpha_record = cast(*B.Struct) decl;
            decl.output_name = "AlphaSample";
        } else {
            assert(parent.name == "beta");
            beta_record = cast(*B.Struct) decl;
            decl.output_name = "BetaSample";
        }
    }

    if decl.kind == .FUNCTION {
        functions += 1;

        if decl.name == "adjust" {
            if parent.name == "alpha" {
                assert(decl.type.type_of_function.return_type.type_of_struct == alpha_record);
                if unique_names decl.output_name = "alpha_adjust";
            } else {
                assert(parent.name == "beta");
                assert(decl.type.type_of_function.arguments[0].type.type_of_struct == beta_record);
                if unique_names decl.output_name = "beta_adjust";
            }
        } else if decl.name == "repeat" decl.output_name = "alpha_repeat";
        else if decl.name == "identity" decl.output_name = "alpha_identity";
        else if decl.name == "long_identity" decl.output_name = "alpha_long_identity";
        else if decl.name == "invoke" decl.output_name = "alpha_invoke";
        else if decl.name == "_source_identity" {
            assert(decl.foreign_name == decl.name);
            decl.output_name = "alpha_c_identity";
        } else {
            assert(decl.name == "scale" && parent.name == "detail");
            decl.output_name = "alpha_scale";
        }
    }

    return .RECURSE;
}

capture_error :: (message: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR {
        errors += 1;
        if contains(message, "cyclic or excessive output namespace graph") cycle_error = true;
    }
}

main :: () {
    defer array_free(cycle_children);
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-cpp-scopes.h", #filepath));
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_cpp_scopes", identifier = "native" });
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.source_files);
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    defer array_free(options.extra_clang_arguments);
    options.visitor = customize;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-cpp-scopes-generated.jai", #filepath), log_results = false));
    options.header = tprint("#include \"%bindings-cpp-scopes.h\"\n", #filepath);
    assert(B.generate_bridge(options, tprint("%../../../../target/developer-tools-cpp-scopes-bridge.cpp", #filepath)));
    assert(namespaces == 6 && records == 4 && functions == 16);
    // Duplicate output names are repaired and nested scopes are qualified,
    // rather than rejected.
    path := tprint("%../../../../target/developer-tools-cpp-scopes-bridge-variants.cpp", #filepath);

    unique_names = false;
    assert(B.generate_bridge(options, path));
    text, ok := read_entire_file(path);

    assert(ok && contains(text, "(Binding_adjust_Result *") && contains(text, "adjust_1("));
    free(text);

    unique_names = true;
    flatten = false;
    assert(B.generate_bridge(options, path));
    text, ok = read_entire_file(path);
    assert(ok && contains(text, "alpha_alpha_adjust(") && contains(text, "beta_beta_adjust("));
    free(text);

    // A visitor that makes the namespace graph cyclic fails the run and leaves the destination alone.
    path = tprint("%../../../../target/developer-tools-cpp-scopes-preserved.jai", #filepath);

    assert(write_entire_file(path, "preserve"));
    flatten = true;
    make_cycle = true;

    push_context {
        context.logger = capture_error;
        assert(!B.generate_bridge(options, path));
    }

    assert(cycle_error);
    make_cycle = false;

    preserved, preserved_ok := read_entire_file(path);

    defer free(preserved);
    assert(errors > 0 && preserved_ok && preserved == "preserve");

    // A record with a default member initializer is still an ordinary record.
    options.source_files[0] = tprint("%bindings-cpp-default-field.h", #filepath);
    options.visitor = null;
    assert(B.generate_bridge(options, path));
    print("ok\n");
}
