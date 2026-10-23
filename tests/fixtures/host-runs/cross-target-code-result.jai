// A plain #run that passes Code in and returns it names declarations of the program, which the
// host twin cannot hand to the target program: in a cross build the run stays on the target
// compiler.
#import "Basic";
C :: #import "Compiler";

copy :: (code: Code) -> Code {
    root := C.compiler_get_nodes(code);
    return C.compiler_get_code(root, code);
}

main :: () {
    #insert #run copy(#code {
        answer := 42;
        assert(answer == 42);
    });
}
