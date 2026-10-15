#import "Basic";
B :: #import "Bindings_Generator";
P :: #import "POSIX";

main :: () {
    directory := to_c_string(tprint("%../../target/generator-platform-copy-work", #filepath));

    defer free(directory);
    assert(P.chdir(directory) == 0);
    B.ensure_local_libclang_exists(tprint("%../../target/generator-platform-copy-source/original.bin", #filepath));
    B.ensure_local_libclang_exists();
    print("ok\n");
}
