// A module `using,map` survives serialization and keeps its renamed names.
#import "Compiler";
#import "Basic";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

main :: () {
    #insert #run rebuild(#code {
        M :: #import,string "value :: () -> int { return 7; }";
        using,map(renamed = value) M;
        assert(renamed() == 7, "the mapped name reaches the original member");
    });
}
