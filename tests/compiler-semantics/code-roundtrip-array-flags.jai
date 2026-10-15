#import "Basic";
#import "Compiler";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

main :: () {
    #insert #run rebuild(#code {
        grow: [..] u32;

        defer array_free(grow);
        array_add(*grow, cast(u32) 42);
        assert(grow.count == 1 && grow[0] == 42);
        fixed: [2] int = .[7, 9];
        view: [] int = fixed;
        view[0] = 13;
        assert(fixed[0] == 13 && view.count == 2);
        Distinct :: #type,distinct s64;
        Related :: #type,isa s64;
        plain: #type s64 = 3;
        distinct: Distinct = cast(Distinct) 4;
        related: Related = cast(Related) 5;

        assert(type_of(distinct) != s64 && type_of(related) != s64);
        assert(cast(s64) distinct == 4 && cast(s64) related == 5 && plain == 3);

        consume :: (arrays: ..[..] u32) -> int {
            total: int;
            for arrays total += it.count;
            return total;
        }

        assert(consume(grow, grow) == 2);
    });

    #insert #run -> Code {
        code := #code {
            edited: [] int;
            defer array_free(edited);
            array_add(*edited, 29);
            assert(edited.count == 1 && edited[0] == 29);
        };

        root := cast(*Code_Block) compiler_get_nodes(code);
        declaration := cast(*Code_Declaration) root.statements[0];

        assert(cast(u32) declaration.type_inst.inst_flags == 0x40);
        declaration.type_inst.inst_flags = .RESIZABLE;

        return compiler_get_code(root, code);
    };

    #insert #run -> Code {
        code := #code {
            fixed: [2] int;
            fixed[2] = 37;
            assert(fixed.count == 3 && fixed[2] == 37);
        };

        root := cast(*Code_Block) compiler_get_nodes(code);
        declaration := cast(*Code_Declaration) root.statements[0];
        dimension := cast(*Code_Literal) declaration.type_inst.array_dimension;
        dimension._s64 = 3;

        return compiler_get_code(root, code);
    };

    Alias :: #insert #run -> Code {
        code := #code #type,distinct s64;
        root := cast(*Code_Type_Instantiation) compiler_get_nodes(code);

        assert(cast(u32) root.inst_flags == 0x10);
        root.inst_flags = .TYPE_DIRECTIVE;

        return compiler_get_code(root, code);
    };

    assert(Alias == s64);
}
