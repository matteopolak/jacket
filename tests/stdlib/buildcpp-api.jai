// BuildCpp: typechecks the public API and builds a tiny C program with the host C compiler.
#import "Basic";
#import "BuildCpp";
#import "File";
#import "Process";

main :: () {
    type: Build_Type = .STATIC_LIBRARY;
    build_static  := build_cpp_static_lib;
    build_dynamic := build_cpp_dynamic_lib;
    build_exe     := build_cpp_executable;

    files := enum_cpp_files("does-not-exist");
    assert(files.count == 0);
    free_cpp_files(files);

    source := "/tmp/jaic_buildcpp_test.c";
    exe    := "/tmp/jaic_buildcpp_test_exe";
    assert(write_entire_file(source, "#include <stdio.h>\nint main(void) { puts(\"from c\"); return 0; }\n"));
    ok := build_cpp(exe, source, type = .EXECUTABLE);
    assert(ok);
    result, output := run_command(exe, capture_and_return_output = true);
    assert(result.exit_code == 0);
    assert(output == "from c\n");
    file_delete(exe);
    file_delete(source);
    print("ok\n");
}
