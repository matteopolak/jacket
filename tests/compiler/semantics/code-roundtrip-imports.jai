#import "Basic";
#import "Compiler";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

main :: () {
    #insert #run rebuild(#code {
        M :: #import,string "#module_parameters(AMOUNT:int); Data :: struct { value:int; } value :: () -> int {return AMOUNT;} hidden :: () -> int {return -1;}"(AMOUNT = 7);
        Alias :: M.Data;
        record: Alias;
        record.value = 19;
        assert(type_of(record) == M.Data && record.value == 19);

        {
            using,only(value) M;
            assert(value() == 7);
            #assert !#exists(hidden);
        }

        {
            using,except(hidden) M;
            assert(value() == 7);
            #assert !#exists(hidden);
        }
    });
}
