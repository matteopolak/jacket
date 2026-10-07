// `push_context,defer_pop ctx;` holds the context for the rest of the block.
// rules: ctx.7 flow.21
#import "Basic";


main :: () {
    ctx := context;
    ctx.thread_index = 7;
    before := context.thread_index;
    {
        push_context, defer_pop ctx;
        if context.thread_index != 7 exit(1);
        x := 3;
        if x + cast(s64) context.thread_index != 10 exit(2);
    }
    if context.thread_index != before exit(3);
    print("ok\n");
}

// rules: ctx.14
// Without a context, both forms push a copy of the current one: the block's changes stay inside it.
bare_forms :: () {
    before := context.thread_index;
    {
        push_context, defer_pop;
        context.thread_index = 77;
        assert(context.thread_index == 77);
    }
    assert(context.thread_index == before);
    push_context {
        context.thread_index = 78;
    }
    assert(context.thread_index == before);
}

// In a #c_call procedure, which has no context, the bare form pushes a default one.
c_callback :: (x: s32) -> s32 #c_call {
    push_context, defer_pop;
    context.thread_index = 5;
    return x + cast(s32) context.thread_index;
}

#run {
    bare_forms();
    assert(c_callback(1) == 6);
}
