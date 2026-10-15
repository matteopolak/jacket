// Reviewed module using,map remains rejected after serialization.
#import "Compiler";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

main :: () {
    #insert #run rebuild(#code {
        M :: #import,string "value :: () -> int { return 7; }";
        using,map(value = renamed) M;
    });
}
