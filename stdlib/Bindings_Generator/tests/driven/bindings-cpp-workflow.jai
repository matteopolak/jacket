#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";
BL :: #import "Bindings_Generator"(TARGET_OS = .LINUX);
BW :: #import "Bindings_Generator"(TARGET_OS = .WINDOWS);

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-cpp-workflow.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_cpp_workflow", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    assert(B.generate_bindings(options, tprint("%../../../../target/developer-tools-cpp-workflow-generated.jai", #filepath), log_results = false));

    assert(generate_linux());
    assert(generate_windows());

    options.header = tprint("#include \"%bindings-cpp-workflow.h\"\n", #filepath);
    assert(B.generate_bridge(options, tprint("%../../../../target/developer-tools-cpp-workflow.cpp", #filepath)));
    nonpod_path := tprint("%../../../../target/developer-tools-cpp-nonpod-generated.jai", #filepath);

    options.source_files[0] = tprint("%bindings-cpp-nonpod.h", #filepath);
    options.header = "";
    assert(B.generate_bindings(options, nonpod_path, log_results = false));

    nonpod, nonpod_ok := read_entire_file(nonpod_path);

    defer free(nonpod);
    // A by-value result with a non-trivial destructor keeps its function and marks the non-POD return.
    assert(nonpod_ok && contains(nonpod, "original_nonpod_value") && contains(nonpod, "#cpp_return_type_is_non_pod"));
    print("ok\n");
}

generate_linux :: () -> bool {
    options: BL.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-cpp-workflow.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, BL.Library.{ filename = "developer_tools_cpp_workflow", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    options.cpu = .X64;

    return BL.generate_bindings(options, tprint("%../../../../target/developer-tools-cpp-workflow-linux.jai", #filepath), log_results = false);
}

generate_windows :: () -> bool {
    options: BW.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-cpp-workflow.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, BW.Library.{ filename = "developer_tools_cpp_workflow", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.library_search_paths, tprint("%../../../../target", #filepath));
    defer array_free(options.library_search_paths);
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    options.log_stripped_declarations = false;
    options.cpu = .X64;
    options.strip_flags = .FUNCTIONS_WITH_VALIST;
    options.visitor = bind_windows_library;

    return BW.generate_bindings(options, tprint("%../../../../target/developer-tools-cpp-workflow-windows.jai", #filepath), log_results = false);
}

// The host library cannot resolve Microsoft-mangled names, so bind each function to the configured library directly.
bind_windows_library :: (decl: *BW.Declaration, _parent: *BW.Declaration) -> BW.Declaration_Visit_Result {
    if decl && decl.kind == .FUNCTION && !decl.library && context.generator.libraries.count > 0 decl.library = xx context.generator.libraries[0];
    return .RECURSE;
}
