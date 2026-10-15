#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    options: B.Generate_Bindings_Options;

    array_add(*options.source_files, tprint("%bindings-cpp-workflow.h", #filepath));
    defer array_free(options.source_files);
    array_add(*options.libraries, B.Library.{ filename = "developer_tools_cpp_workflow", identifier = "native" });
    defer array_free(options.libraries);
    array_add(*options.extra_clang_arguments, "-x", "c++", "-std=c++17");
    defer array_free(options.extra_clang_arguments);
    options.add_generator_command = false;
    assert(B.generate_bindings(options, tprint("%../../target/developer-tools-cpp-workflow-generated.jai", #filepath), log_results = false));

    for target_os: type_of(OS).[.LINUX, .WINDOWS] {
        options.os = target_os;
        options.cpu = .X64;
        suffix := ifx target_os == .WINDOWS then "windows" else "linux";
        assert(B.generate_bindings(options, tprint("%../../target/developer-tools-cpp-workflow-%.jai", #filepath, suffix), log_results = false));
    }

    options.os = OS;
    options.cpu = CPU;
    options.header = tprint("#include \"%bindings-cpp-workflow.h\"\n", #filepath);
    assert(B.generate_bridge(options, tprint("%../../target/developer-tools-cpp-workflow.cpp", #filepath)));
    print("ok\n");
}
