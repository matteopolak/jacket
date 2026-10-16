#import "Basic";
C :: #import "Compiler";

copy :: (code: Code) -> Code {
    root := C.compiler_get_nodes(code);
    return C.compiler_get_code(root, code);
}

main :: () {
    bare :: #run copy(#code value := 2);

    #insert bare;
    assert(value == 2);
    scoped :: #run copy(#code {
        value := 9;
        assert(value == 9);
    });

    #insert scoped;
    assert(value == 2);
    print("roundtrip ok\n");
}
