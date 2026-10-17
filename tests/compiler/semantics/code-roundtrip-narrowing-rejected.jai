#import "Compiler";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

PI :: #insert #run rebuild(#code 3.14159265358979);

main :: () {
    narrowed: float32 = PI;
}
