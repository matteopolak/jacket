#import "Basic";
C :: #import "Compiler";

// `a + b` becomes `c * 5`: a renamed identifier, a new operator and a node the program made.
product :: (source: Code) -> Code {
    root := cast(*C.Code_Binary_Operator) C.compiler_get_nodes(source);

    assert(root.kind == .BINARY_OPERATOR);
    (cast(*C.Code_Ident) root.left).name = "c";
    root.operator_type = #char "*";
    five := New(C.Code_Literal);
    five.value_type = .NUMBER;
    five._s64 = 5;
    root.right = five;

    return C.compiler_get_code(root, source);
}

// `1` becomes the float `2.5`.
float :: (source: Code) -> Code {
    root := cast(*C.Code_Literal) C.compiler_get_nodes(source);

    assert(root.kind == .LITERAL);
    root._float64 = 2.5;
    root.value_flags |= .FLOAT;

    return C.compiler_get_code(root, source);
}

// The two statements of a block swap places.
swapped :: (source: Code) -> Code {
    root := cast(*C.Code_Block) C.compiler_get_nodes(source);

    assert(root.kind == .BLOCK && root.statements.count == 2);
    first := root.statements[0];

    root.statements[0] = root.statements[1];
    root.statements[1] = first;

    return C.compiler_get_code(root, source);
}

main :: () {
    a, b, c := 6, 7, 3;
    p := #insert #run product(#code a + b);
    f := #insert #run float(#code 1);

    assert(p == 15 && f == 2.5);
    x := 0;

    #insert #run swapped(#code {
        x = 1;
        x = 2;
    });
    assert(x == 1);
    print("edited nodes ok\n");
}
