// Checks the claims of docs/language/context.md, one group per rule.
// rules: ctx.1 ctx.2 ctx.3 ctx.4 ctx.5 ctx.6 ctx.7 ctx.10 ctx.13
#import "Basic";

#add_context depth: int = 7;
#add_context tag: string = "base";

read_depth :: () -> int {
    return context.depth;
}

bump :: () {
    context.depth += 1;
}

depth_in_push :: (target: int) -> int {
    ctx := context;
    ctx.depth = target;
    push_context ctx {
        return read_depth();
    }
    return -1;
}

c_side :: () -> int #c_call {
    fresh: #Context;
    push_context fresh {
        return context.depth;
    }
}

main :: () {
    // ctx.1: the context carries allocator, logger, temporary storage, thread index and added fields.
    assert(context.allocator.proc != null, "allocator");
    assert(context.logger != null, "logger");
    assert(context.temporary_storage != null, "temporary storage");
    #assert type_of(context.thread_index) == u32;
    assert(context.tag == "base", "added field");

    // ctx.3: #add_context adds a field with its default.
    assert(context.depth == 7, "default of an added field");

    // ctx.2, ctx.4: push_context installs a modified copy for its block only.
    {
        ctx := context;
        ctx.depth = 99;
        push_context ctx {
            assert(read_depth() == 99, "callee sees the pushed copy");
        }
        assert(ctx.depth == 99, "the copy itself is unchanged");
        assert(read_depth() == 7, "restored after the block");
    }

    // ctx.5: assignments write through the active context, so callees see them.
    bump();
    assert(context.depth == 8, "callee's write is visible");
    context.depth += 1;
    assert(read_depth() == 9, "caller's write is visible to callees");
    context.depth = 7;

    // ctx.6: push_context restores on return, break and continue.
    assert(depth_in_push(50) == 50, "return from inside push_context");
    assert(context.depth == 7, "restored after return");
    for 1..3 {
        ctx := context;
        ctx.depth = 100 + it;
        push_context ctx {
            if it == 1 continue;
            if it == 2 break;
        }
    }
    assert(context.depth == 7, "restored after continue and break");

    // ctx.7: push_context,defer_pop keeps the context until the end of the enclosing block.
    {
        ctx := context;
        ctx.depth = 31;
        push_context, defer_pop ctx;
        assert(read_depth() == 31, "held for the rest of the block");
    }
    assert(read_depth() == 7, "popped at the end of the block");

    // ctx.10: a #c_call body can establish a context with push_context.
    assert(c_side() == 7, "#Context defaults inside #c_call");

    // ctx.13: indirect calls pass the active context too.
    reader := read_depth;
    {
        ctx := context;
        ctx.depth = 64;
        push_context ctx {
            assert(reader() == 64, "indirect call sees the pushed context");
        }
    }

    print("ok\n");
}
