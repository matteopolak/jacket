// test: only(.WINDOWS) skip_on(.WASM) compile_fail
#import "Basic";
B :: #import "BuildCpp";
F :: #import "File";
S :: #import "String";

#run {
    directory := sprint("%/../../../../target/portable-tests/cpp-wrong-machine", S.path_strip_filename(#file));

    defer free(directory);
    assert(F.make_directory_if_it_does_not_exist(directory, true));
    source := sprint("%/valid.c", directory);

    defer free(source);
    assert(F.write_entire_file(source, "int answer(void) { return 42; }\n"));
    #if CPU == .ARM64 triple :: "x86_64-pc-windows-msvc";
    else triple :: "aarch64-pc-windows-msvc";
    flags := string.["-target", triple];
    output := sprint("%/wrong", directory);

    defer free(output);
    assert(!B.build_cpp(output, source, type = .OBJ_FILE, extra = flags, compiler_executable_path = "clang"));
}

main :: () {}
