// BuildCpp: typechecks the public API, finds sources with enum_cpp_files, and builds a
// tiny C program, a static library and a dynamic library with the host C compiler.
#import "Basic";
#import "BuildCpp";
#import "File";
#import "Process";
#import "File_Utilities";
#import "String";

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

    dir := "/tmp/jaic_buildcpp_lib";
    delete_directory(dir);
    assert(make_directory_if_it_does_not_exist(tprint("%/nested", dir), recursive = true));
    assert(write_entire_file(tprint("%/add.c", dir), "int add(int a, int b) { return a + b; }\n"));
    assert(write_entire_file(tprint("%/add.h", dir), "int add(int a, int b);\n"));
    assert(write_entire_file(tprint("%/nested/more.cpp", dir), "extern \"C\" int twice(int a) { return a * 2; }\n"));

    shallow := enum_cpp_files(dir);
    assert(shallow.count == 1 && ends_with(shallow[0], "add.c"));
    free_cpp_files(shallow);
    deep := enum_cpp_files(dir, recursive = true);
    assert(deep.count == 2);
    free_cpp_files(deep);

    // Objects are made in the working directory and removed once archived.
    assert(build_cpp_static_lib(tprint("%/libadd", dir), tprint("%/add.c", dir), working_directory = dir));
    assert(file_exists(tprint("%/libadd.a", dir)));
    assert(!file_exists(tprint("%/add.o", dir)));

    assert(build_cpp_dynamic_lib(tprint("%/libadd", dir), tprint("%/add.c", dir), working_directory = dir));
    #if OS == .MACOS  assert(file_exists(tprint("%/libadd.dylib", dir)));
    else              assert(file_exists(tprint("%/libadd.so", dir)));

    // OBJ_FILE keeps the object.
    assert(build_cpp("unused", tprint("%/add.c", dir), type = .OBJ_FILE, working_directory = dir));
    assert(file_exists(tprint("%/add.o", dir)));

    delete_directory(dir);
    print("ok\n");
}
