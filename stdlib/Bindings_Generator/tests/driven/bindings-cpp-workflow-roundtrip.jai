#import "Basic";
G :: #import "developer-tools-cpp-workflow-generated";

// The generated bindings describe the C++ declarations; the bridge's out-pointer ABI is declared separately.
adapters :: #foreign_library "../../../../target/developer_tools_cpp_workflow";

adapter_pair :: (result: *G.Original_Cpp_Pair, number: s32, scale: float64) #foreign adapters "original_cpp_workflow_pair";
adapter_empty :: (result: *G.Original_Cpp_Pair) #foreign adapters "original_cpp_workflow_empty";
adapter_collision :: (result: *G.Original_Cpp_Pair, out_result_0: s32) #foreign adapters "original_cpp_workflow_collision";
adapter_small :: (result: *G.Original_Cpp_Small, value: G.Original_Cpp_Small) #foreign adapters "original_cpp_workflow_small";
adapter_large :: (result: *G.Original_Cpp_Large, value: G.Original_Cpp_Large) #foreign adapters "original_cpp_workflow_large";

main :: () {
    assert(G.original_cpp_workflow_sum(left = 17, right = 25) == 42);
    assert(G.original_cpp_workflow_scale(value = 1.25, factor = 4.0) == 5.0);
    value: s32 = 41;

    G.original_cpp_workflow_bump(value = *value);
    assert(value == 42);
    #if OS == .WINDOWS assert(G.original_cpp_workflow_long(value = 41) == 42);
    else assert(G.original_cpp_workflow_long(value = 0x100000001) == 0x100000002);
    pair: G.Original_Cpp_Pair;

    adapter_pair(*pair, number = 40, scale = 1.5);
    assert(pair.number == 40 && pair.scale == 1.5);
    assert(G.original_cpp_workflow_pair_sum(pair) == 41.5);
    reference := G.original_cpp_workflow_reference(*pair);

    assert(reference == *pair && pair.number == 41 && reference.scale == 1.5);
    constant_reference := G.original_cpp_workflow_const_reference(*pair);

    assert(constant_reference == *pair && constant_reference.number == 41);
    adapter_empty(*pair);
    assert(pair.number == 42 && pair.scale == 2.5);
    adapter_collision(*pair, out_result_0 = 17);
    assert(pair.number == 17 && pair.scale == 3.5);
    small := G.Original_Cpp_Small.{ left = 17, right = 25 };
    small_result: G.Original_Cpp_Small;

    adapter_small(*small_result, small);
    assert(small_result.left == 25 && small_result.right == 17);
    large := G.Original_Cpp_Large.{ first = 1.25, second = 2.5, third = 5.0 };
    large_result: G.Original_Cpp_Large;

    adapter_large(*large_result, large);
    assert(large_result.first == 5.0 && large_result.second == 1.25 && large_result.third == 2.5);
    print("ok\n");
}
