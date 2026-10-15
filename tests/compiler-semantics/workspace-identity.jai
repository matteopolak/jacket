// Canonical preloaded #compiler primitive, in global and procedure-local #run.
#import "Basic";
C :: #import "Compiler";

root_workspace :: #run get_current_workspace();
first_child :: #run C.compiler_create_workspace("empty-first");
second_child :: #run C.compiler_create_workspace("empty-second");

selected_in_procedure :: () -> Workspace {
    selected :: #run get_current_workspace();
    return selected;
}

main :: () {
    assert(selected_in_procedure() == root_workspace);
    assert(root_workspace != first_child && root_workspace != second_child);
    assert(first_child != second_child);
    print("ok\n");
}
