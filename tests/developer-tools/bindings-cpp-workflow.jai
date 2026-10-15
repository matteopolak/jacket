#import "Basic";
#import "File";
#import "String";
B :: #import "Bindings_Generator";

rejection_seen: bool;

capture :: (text: string, _data: *void, info: Log_Info) {
    if info.common_flags & .ERROR && contains(text, "non-POD C++ record") rejection_seen = true;
}

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
    failed_path := tprint("%../../target/developer-tools-cpp-nonpod-generated.jai", #filepath);

    assert(write_entire_file(failed_path, "original nonpod destination\n"));
    options.source_files[0] = tprint("%bindings-cpp-nonpod.h", #filepath);
    options.header = "";

    push_context {
        context.logger = capture;
        assert(!B.generate_bindings(options, failed_path, log_results = false));
    }

    preserved, preserved_ok := read_entire_file(failed_path);

    defer free(preserved);
    assert(rejection_seen && preserved_ok && preserved == "original nonpod destination\n");
    print("ok\n");
}
