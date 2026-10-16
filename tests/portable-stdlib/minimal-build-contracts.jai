// Real child workspace; no CLI replacement or private implementation access.
#import "Basic";
M :: #import "Minimal_Metaprogram";
C :: #import "Compiler";

#run,stallable {
    M.build();
    options := C.get_build_options();

    assert(!options.write_added_strings);
    directory := get_working_directory();

    write_strings("cwd=", directory, "\n");
    free(directory);
}
