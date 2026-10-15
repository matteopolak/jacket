#import "Basic";
#import "Compiler";

rebuild :: (code: Code) -> Code {
    return compiler_get_code(compiler_get_nodes(code), code);
}

main :: () {
    #insert #run rebuild(#code {
        Choice :: enum {
            FIRST :: 7;
            SECOND :: 19;
        }

        take :: (value: Choice) -> int {
            return cast(int) value;
        }

        inferred := take(.FIRST);
        qualified := take(Choice.SECOND);
        value: Choice = .SECOND;

        assert(inferred == 7 && qualified == 19 && value == .SECOND);
    });

    Choice :: enum {
        FIRST :: 7;
        SECOND :: 19;
    }

    take :: (value: Choice) -> int {
        return cast(int) value;
    }

    edited := #insert #run -> Code {
        code := #code take(.FIRST);
        root, nodes := compiler_get_nodes(code);
        changed: bool;

        for nodes if it.kind == .IDENT {
            ident := cast(*Code_Ident) it;
            if ident.name == "FIRST" {
                ident.name = "SECOND";
                changed = true;
            }
        }

        assert(changed);

        return compiler_get_code(root, code);
    };

    assert(edited == 19);
}
