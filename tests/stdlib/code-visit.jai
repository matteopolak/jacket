// Code_Visit walks the syntactic children of compiler_get_nodes trees.
#import "Basic";
#import "Compiler";
#import "Code_Visit";

count_calls :: (code: Code) -> int, int {
    root := compiler_get_nodes(code);
    breadth: [..] *Code_Node;
    get_recursive_nodes_breadth_first(root, *breadth);
    calls := 0;
    for breadth if it.kind == .PROCEDURE_CALL calls += 1;
    depth: [..] *Code_Node;
    get_recursive_nodes_depth_first(root, *depth);
    return calls, depth.count;
}

#run {
    calls, total := count_calls(#code {
        x := f(g(1), 2 + h());
        if x > 3 { print("%\n", x); }
    });
    assert(calls == 4, "calls: %", calls);
    assert(total > 10, "nodes: %", total);
}

// Several targets take one value, types included (`T1, T2 = bool;` in a #modify).
#run {
    a, b: Type;
    a, b = bool;
    assert(a == bool && b == bool);
}

main :: () {}
