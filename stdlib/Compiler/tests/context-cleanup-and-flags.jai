// Original public AST flag and macro context-lifetime regression.
#import "Basic";
C :: #import "Compiler";

#add_context marker: s64 = 7;

push_copy :: () #expand {
    copied := context;
    `push_context, defer_pop copied;
}

#run {
    code := #code {
        `push_context, defer_pop;
        `push_context {}
    }

    root, expressions := C.compiler_get_nodes(code);
    found := false;
    found_block := false;

    for expression: expressions {
        if expression.kind == .PUSH_CONTEXT {
            pushed := cast(*C.Code_Push_Context) expression;

            assert(pushed.to_push != null);

            if pushed.block != null {
                assert((cast(u32) pushed.push_context_flags) == 1);
                found_block = true;
            } else {
                assert((cast(u32) pushed.push_context_flags) == 3);
                found = true;
            }
        }
    }

    assert(root != null && found && found_block);
}

main :: () {
    context.marker = 55;

    {
        push_copy();
        context.marker = 91;
    }

    assert(context.marker == 55);
    print("ok\n");
}
