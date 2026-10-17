C :: #import "Compiler";

#placeholder Body;
body_scope :: #code Body;

#run {
    workspace := C.compiler_create_workspace("Another workspace");
    defer C.compiler_destroy_workspace(workspace);
    C.add_build_string("Body :: \"return 42;\";", workspace, body_scope);
}

main :: () {}
