// test: only(.WINDOWS) skip_on(.WASM)
#import "Basic";
B :: #import "BuildCpp";
F :: #import "File";
U :: #import "File_Utilities";
S :: #import "String";
P :: #import "Process";
W :: #import "Windows";
Wide :: #import "Windows_Utf8";

run_contract :: () {
    assert(B.is_native_msvc(B.default_compiler(false)), "this fixture requires the installed MSVC toolset");
    root := sprint("%/../../../target/portable-tests/cpp windows build, café", S.path_strip_filename(#file));

    defer free(root);
    assert(F.make_directory_if_it_does_not_exist(root, true));
    source := sprint("%/library source.c", root);

    defer free(source);
    assert(F.write_entire_file(source, "#include <stdlib.h>\n#ifndef BUILDCPP_FLAG\n#error missing extra argument\n#endif\n__declspec(dllexport) int answer(void) { return abs(-42); }\n"));
    flags := string.["/DBUILDCPP_FLAG=1"];
    library := sprint("%/answer", root);

    defer free(library);
    // Rebuild the archive too: stale members and linker side products must not survive.
    other := sprint("%/old member.c", root);

    defer free(other);
    assert(F.write_entire_file(other, "int old_member(void) { return 7; }\n"));
    assert(B.build_cpp_static_lib(library, source, other, extra = flags));
    assert(B.build_cpp_static_lib(library, source, extra = flags));
    archive_result, listing, archive_error, archive_timeout := P.run_command(B.default_archiver(), "/LIST", tprint("%.lib", library), capture_and_return_output = true);

    defer free(listing);
    defer free(archive_error);
    assert(archive_result.type == .EXITED && archive_result.exit_code == 0 && !archive_timeout);
    assert(S.contains(listing, "0.obj") && !S.contains(listing, "1.obj"));

    entry := sprint("%/main source.cpp", root);

    defer free(entry);
    assert(F.write_entire_file(entry, "#include <stdio.h>\nextern \"C\" int answer(void);\nint main() { printf(\"answer %d\\n\", answer()); return 0; }\n"));
    libraries := string.[tprint("%.lib", library)];
    executable := sprint("%/static program", root);

    defer free(executable);
    assert(B.build_cpp_executable(executable, entry, library_files = libraries));
    check_program(tprint("%.exe", executable));
    assert(B.build_cpp_dynamic_lib(library, source, debug = true, extra = flags));
    assert(U.file_exists(tprint("%.pdb", library)));
    dll := sprint("%.dll", library);

    defer free(dll);
    wide, converted := Wide.utf8_to_wide(dll);

    defer free(wide);
    assert(converted);
    handle := W.LoadLibraryW(wide);

    assert(handle != null);
    Answer :: #type () -> s32 #c_call;
    symbol := to_c_string("answer");

    defer free(symbol);
    answer := cast(Answer) W.GetProcAddress(handle, symbol);

    assert(answer != null && answer() == 42);
    assert(W.FreeLibrary(handle) != 0);
    // The same C++ entry now links through the DLL import library.
    assert(B.build_cpp_executable(executable, entry, library_files = libraries));
    check_program(tprint("%.exe", executable));
    // Explicit paths use the same argument-array execution as discovered tools.
    assert(B.build_cpp("single object", "library source.c", type = .OBJ_FILE, working_directory = root, extra = flags, compiler_executable_path = B.default_compiler(false)));
    assert(U.file_exists(tprint("%/single object.obj", root)));
    assert(B.build_cpp("clang-cl object", "library source.c", type = .OBJ_FILE, working_directory = root, extra = flags, compiler_executable_path = B.default_clang_compiler(false)));
    assert(B.build_cpp("clang object", "library source.c", type = .OBJ_FILE, working_directory = root, extra = string.["-DBUILDCPP_FLAG=1"], compiler_executable_path = "clang"));
    assert(U.visit_files(root, false, cast(*void) null, check_cleanup, visit_directories = true));
}

main :: () {
    print("ok\n");
}

WINDOWS_HOST :: #run,host -> bool {
    return OS == .WINDOWS;
};

#if WINDOWS_HOST #run run_contract();

check_cleanup :: (info: *U.File_Visit_Info, data: *void) {
    assert(!S.contains(info.full_name, ".build-"));
}

check_program :: (executable: string) {
    result, output, errors, timed := P.run_command(executable, capture_and_return_output = true);
    defer free(output);
    defer free(errors);
    assert(result.type == .EXITED && result.exit_code == 0 && !timed && output == "answer 42\r\n" && errors == "");
}
