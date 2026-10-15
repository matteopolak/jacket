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
    pair: G.Original_Cpp_Pair;

    G.original_cpp_workflow_pair(*pair, number = 40, scale = 1.5);
    assert(pair.number == 40 && pair.scale == 1.5);
    assert(G.original_cpp_workflow_pair_sum(pair) == 41.5);
    reference := G.original_cpp_workflow_reference(*pair);

    assert(reference == *pair && pair.number == 41 && reference.scale == 1.5);
    constant_reference := G.original_cpp_workflow_const_reference(*pair);

    assert(constant_reference == *pair && constant_reference.number == 41);
    G.original_cpp_workflow_empty(*pair);
    assert(pair.number == 42 && pair.scale == 2.5);
    G.original_cpp_workflow_collision(*pair, out_result_0 = 17);
    assert(pair.number == 17 && pair.scale == 3.5);
    small := G.Original_Cpp_Small.{ left = 17, right = 25 };
    small_result: G.Original_Cpp_Small;

    G.original_cpp_workflow_small(*small_result, small);
    assert(small_result.left == 25 && small_result.right == 17);
    large := G.Original_Cpp_Large.{ first = 1.25, second = 2.5, third = 5.0 };
    large_result: G.Original_Cpp_Large;

    G.original_cpp_workflow_large(*large_result, large);
    assert(large_result.first == 5.0 && large_result.second == 1.25 && large_result.third == 2.5);
    print("ok\n");
}
