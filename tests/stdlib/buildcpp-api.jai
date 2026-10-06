// BuildCpp: typechecks the public API, finds sources with enum_cpp_files, and builds a
// tiny C program, a static library and a dynamic library with the host C compiler.
// jailint: allow-file(unused_variable) -- the declarations only type-check the API.
#import "Basic";
#import "BuildCpp";
#import "File";
#import "Process";
#import "File_Utilities";
#import "String";

main :: () {
    type: Build_Type = .STATIC_LIBRARY;
    build_static := build_cpp_static_lib;
    build_dynamic := build_cpp_dynamic_lib;
    build_exe := build_cpp_executable;

    files := enum_cpp_files("does-not-exist");
    assert(files.count == 0);
    free_cpp_files(files);

    // The WASM target (the browser playground) starts no processes, so it has no C compiler:
    // there only the source search runs; the build calls are still compiled.
    can_compile := OS != .WASM;

    if can_compile {
        // On Windows /tmp is \tmp on the current drive, which need not exist yet.
        make_directory_if_it_does_not_exist("/tmp");
        source := "/tmp/jaic_buildcpp_test.c";
        exe := "/tmp/jaic_buildcpp_test_exe";
        assert(write_entire_file(source, "#include <stdio.h>\nint main(void) { puts(\"from c\"); return 0; }\n"));
        ok := build_cpp(exe, source, type = .EXECUTABLE);
        assert(ok);
        result, output := run_command(exe, capture_and_return_output = true);
        assert(result.exit_code == 0);
        // The C runtime writes text-mode newlines: "\r\n" on Windows.
        expected := ifx OS == .WINDOWS then "from c\r\n" else "from c\n";
        assert(output == expected, "C program printed '%'", output);
        file_delete(exe);
        file_delete(source);
    }

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

    if can_compile {
        // Windows names objects and libraries for its own toolchain.
        #if OS == .WINDOWS {
            OBJECT :: "obj";
            ARCHIVE :: "lib";
            SHARED :: "dll";
        } else #if OS == .MACOS {
            OBJECT :: "o";
            ARCHIVE :: "a";
            SHARED :: "dylib";
        } else {
            OBJECT :: "o";
            ARCHIVE :: "a";
            SHARED :: "so";
        }
        // Objects are made in the working directory and removed once archived.
        assert(build_cpp_static_lib(tprint("%/libadd", dir), tprint("%/add.c", dir), working_directory = dir));
        assert(file_exists(tprint("%/libadd.%", dir, ARCHIVE)));
        assert(!file_exists(tprint("%/add.%", dir, OBJECT)));

        assert(build_cpp_dynamic_lib(tprint("%/libadd", dir), tprint("%/add.c", dir), working_directory = dir));
        assert(file_exists(tprint("%/libadd.%", dir, SHARED)));

        // OBJ_FILE keeps the object.
        assert(build_cpp("unused", tprint("%/add.c", dir), type = .OBJ_FILE, working_directory = dir));
        assert(file_exists(tprint("%/add.%", dir, OBJECT)));
    }

    delete_directory(dir);
    print("ok\n");
}
