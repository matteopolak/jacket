#import "Basic";
#import "Extensions/Environment";
B :: #import "Bindings_Generator";
P :: #import "POSIX";

main :: () {
    directory := to_c_string(output_path("generator-platform-copy-work"));

    defer free(directory);
    assert(P.chdir(directory) == 0);
    B.ensure_local_libclang_exists(output_path("generator-platform-copy-source/original.bin"));
    B.ensure_local_libclang_exists();
    print("ok\n");
}
