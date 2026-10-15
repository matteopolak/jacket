#import "Basic";
G :: #import "developer-tools-cpp-workflow-generated";

main :: () {
    assert(G.original_cpp_workflow_sum(left = 17, right = 25) == 42);
    assert(G.original_cpp_workflow_scale(value = 1.25, factor = 4.0) == 5.0);
    value: s32 = 41;

    G.original_cpp_workflow_bump(value = *value);
    assert(value == 42);
    #if OS == .WINDOWS assert(G.original_cpp_workflow_long(value = 41) == 42);
    else assert(G.original_cpp_workflow_long(value = 0x100000001) == 0x100000002);
    print("ok\n");
}
