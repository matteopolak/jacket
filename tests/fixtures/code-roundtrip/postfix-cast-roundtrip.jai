#import "Basic";
C :: #import "Compiler";

copy :: (code: Code) -> Code {
    root := C.compiler_get_nodes(code);
    return C.compiler_get_code(root, code);
}

check :: #insert #run copy(#code () -> bool {
    value: s64 = 13;
    replacement: s64 = 29;
    slot := cast(*void) *value;
    raw := cast(*void) *slot;
    offset: s64;

    cast(**void, cast(*void, raw) +offset).* = xx *replacement;
    if cast(*s64, slot).* != 29 return false;

    Pair :: struct {
        left: u32;
    }

    pair: Pair;
    cast(*Pair, cast(*void, *pair)) .left = 41;
    if pair.left != 41 return false;

    cells := u32.[1, 2];
    cast(*u32, cast(*void, cells.data)) [1] = 43;
    if cells[1] != 43 return false;

    Fn :: (x: s64) -> s64;

    plus_one :: (x: s64) -> s64 {
        return x + 1;
    }

    procedure := cast(*void) plus_one;

    return cast(Fn, procedure) (46) == 47;
});

#run {
    w := C.compiler_create_workspace("postfix cast modification");
    options := C.get_build_options(w);
    options.output_type = .NO_OUTPUT;
    C.set_build_options(options, w);
    C.add_build_string("#import \"Basic\"; probe :: () -> bool { value:u32=13; raw:=cast(*void,*value); return cast(*u32,raw).*==13; } main :: () { assert(probe()); }", w);
    C.compiler_begin_intercept(w);
    changed := false;
    prefix := C.compiler_get_nodes(#code assert(true));

    while true {
        message := C.compiler_wait_for_message();

        assert(message != null);

        if message.kind == .TYPECHECKED {
            typed := cast(*C.Message_Typechecked) message;
            for item: typed.procedure_bodies {
                body := item.expression;
                if body.header.name == "probe" && !changed {
                    statements: [..] *C.Code_Node;

                    array_add(*statements, prefix);
                    for statement: body.block.statements array_add(*statements, statement);

                    body.block.statements = statements;
                    C.compiler_modify_procedure(w, body);
                    C.add_build_string("#assert probe();", w);
                    changed = true;
                }
            }
        }

        if message.kind == .COMPLETE {
            assert((cast(*C.Message_Complete) message).error_code == .NONE);
            break;
        }
    }

    assert(changed);
    C.compiler_end_intercept(w);
}

main :: () {
    assert(check());
    print("postfix casts ok\n");
}
