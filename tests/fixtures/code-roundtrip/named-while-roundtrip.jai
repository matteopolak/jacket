#import "Basic";
C :: #import "Compiler";

copy :: (code: Code) -> Code {
    root := C.compiler_get_nodes(code);
    return C.compiler_get_code(root, code);
}

with_condition :: #insert #run copy(#code (body: Code) #expand {
    while `condition_value := cast(u32) 3 {
        #insert body;
    }
});

main :: () {
    observed: u32;

    with_condition(#code {
        observed = condition_value;
        break condition_value;
    });
    assert(observed == 3);
    print("named while roundtrip ok\n");
}
