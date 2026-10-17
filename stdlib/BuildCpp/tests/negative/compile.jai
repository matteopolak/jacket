#import "Basic";
B :: #import "BuildCpp";
F :: #import "File";
S :: #import "String";

#run {
    directory := sprint("%/../../../../target/portable-tests/cpp-failure", S.path_strip_filename(#file));

    defer free(directory);
    assert(F.make_directory_if_it_does_not_exist(directory, true));
    source := sprint("%/broken.c", directory);

    defer free(source);
    output := sprint("%/old", directory);

    defer free(output);
    assert(F.write_entire_file(source, "this is deliberately invalid C;\n"));
    assert(F.write_entire_file(output, "preserved"));
    assert(!B.build_cpp_executable(output, source));
}

main :: () {}
