#import "Basic";
B :: #import "BuildCpp";
F :: #import "File";
U :: #import "File_Utilities";
S :: #import "String";
P :: #import "Process";
Native :: #import "POSIX";

Answer :: #type () -> s32 #c_call;

#run {
    root := sprint("%/../../target/portable-tests/cpp build", S.path_strip_filename(#file));

    defer free(root);
    assert(F.make_directory_if_it_does_not_exist(root, true));
    source := sprint("%/library.c", root);

    defer free(source);
    assert(F.write_entire_file(source, "#ifndef PORTABLE_FLAG\n#error extra compiler argument was lost\n#endif\nint portable_answer(void) { return 42; }\n"));
    flags := string.["-DPORTABLE_FLAG=1"];
    other := sprint("%/other.c", root);

    defer free(other);
    assert(F.write_entire_file(other, "int removed_member(void) { return 8; }\n"));
    library := sprint("%/answer", root);

    defer free(library);
    assert(B.build_cpp_static_lib(library, source, other, debug = true, extra = flags, ar_executable_path = "ar"));
    assert(B.build_cpp_static_lib(library, source, extra = flags));
    library_file := sprint("%.a", library);

    defer free(library_file);
    assert(U.file_exists(library_file));
    archive_result, listing, archive_error, archive_timeout := P.run_command("ar", "-t", library_file, capture_and_return_output = true);

    assert(archive_result.type == .EXITED && archive_result.exit_code == 0 && !archive_timeout && S.ends_with(listing, "0.o\n") && !S.contains(listing, "1.o\n") && archive_error == "");
    free(listing);
    free(archive_error);
    entry := sprint("%/main.cpp", root);

    defer free(entry);
    assert(F.write_entire_file(entry, "#include <stdio.h>\nextern \"C\" int portable_answer(void);\nint main() { printf(\"answer %d\\n\", portable_answer()); return 0; }\n"));
    executable := sprint("%/program", root);

    defer free(executable);
    libraries := string.[library_file];

    assert(B.build_cpp_executable(executable, entry, library_files = libraries));
    result, out, err, timed := P.run_command(executable, capture_and_return_output = true);

    assert(result.type == .EXITED && result.exit_code == 0 && out == "answer 42\n" && err == "" && !timed);
    free(out);
    free(err);
    assert(B.build_cpp_dynamic_lib(library, source, extra = flags));
    #if OS == .MACOS suffix :: ".dylib";
    else suffix :: ".so";
    dynamic := sprint("%%", library, suffix);

    defer free(dynamic);
    assert(U.file_exists(dynamic));
    name := to_c_string(dynamic);

    defer free(name);
    handle := Native.dlopen(name, Native.RTLD_NOW);

    assert(handle != null);
    symbol := to_c_string("portable_answer");

    defer free(symbol);
    answer := cast(Answer) Native.dlsym(handle, symbol);

    assert(answer != null && answer() == 42);
    assert(Native.dlclose(handle) == 0);
    object := sprint("%/single", root);

    defer free(object);
    assert(B.build_cpp("single", "library.c", type = .OBJ_FILE, working_directory = root, compiler_executable_path = "clang", extra = flags));
    object_file := sprint("%.o", object);

    defer free(object_file);
    assert(U.file_exists(object_file));
    files := B.enum_cpp_files(root);

    assert(files.count == 3);
    B.free_cpp_files(files);

    #if OS == .MACOS {
        universal := sprint("%/universal.dylib", root);

        defer free(universal);
        assert(B.build_universal_macos_binary(universal, source, type = .DYNAMIC_LIBRARY, extra = flags));

        for architecture: string.["arm64", "x86_64"] {
            verified, stdout, stderr, expired := P.run_command("lipo", "-verify_arch", architecture, universal, capture_and_return_output = true);
            assert(verified.type == .EXITED && verified.exit_code == 0 && !expired && stdout == "" && stderr == "");
            free(stdout);
            free(stderr);
        }
    }
}

main :: () {
    print("ok\n");
}
