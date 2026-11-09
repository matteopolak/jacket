// test: only(.LINUX,.MACOS,.WINDOWS) compile_fail
#import "Basic";
B :: #import "BuildCpp";
F :: #import "File";
#import "Extensions/Environment";

#run {
    directory := output_path("portable-tests/cpp-missing-tool");

    assert(F.make_directory_if_it_does_not_exist(directory, true));
    source := sprint("%/valid.c", directory);

    defer free(source);
    output := sprint("%/old", directory);

    defer free(output);
    assert(F.write_entire_file(source, "int main(void) { return 0; }\n"));
    assert(!B.build_cpp_executable(output, source, compiler_executable_path = "/missing-portable-c-compiler"));
}

main :: () {}
