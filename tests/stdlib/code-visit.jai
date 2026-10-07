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
        if x > 3 {
            print("%\n", x);
        }
    });
    assert(calls == 4, "calls: %", calls);
    assert(total > 10, "nodes: %", total);
}

// visit_pre_and_postorder enters every node once and leaves it once, after its subtree.
#run {
    root := compiler_get_nodes(#code {
        a := f(b, c);
    });
    depth: [..] *Code_Node;
    get_recursive_nodes_depth_first(root, *depth);
    entered: [..] *Code_Node;
    open := 0;
    deepest := 0;
    left := 0;
    visit_pre_and_postorder(root, #code {
        if preorder {
            array_add(*entered, it);
            open += 1;
            if open > deepest deepest = open;
        } else {
            open -= 1;
            left += 1;
        }
    });
    assert(entered.count == depth.count, "entered % of %", entered.count, depth.count);
    assert(left == depth.count && open == 0);
    assert(deepest > 2, "deepest: %", deepest);
    // Entry order is the depth-first preorder.
    for entered assert(it == depth[it_index], "entry %", it_index);
}

#run {
    // Each expansion declares `it` and `preorder` in the caller's scope, hence one per block.
    root := compiler_get_nodes(#code {
        a := f(b, c);
    });
    first_leaves := -1;
    count := 0;
    visit_pre_and_postorder(root, #code {
        count += 1;
        if !preorder && it == root && first_leaves < 0 first_leaves = count;
    });
    assert(first_leaves == count, "the root leaves last");
    {
        visit_pre_and_postorder(null, #code {
            assert(false);
        });
    }
}

// Several targets take one value, types included (`T1, T2 = bool;` in a #modify).
#run {
    a, b: Type;
    a, b = bool;
    assert(a == bool && b == bool);
}

main :: () {}
