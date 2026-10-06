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
