#import "Basic";

#add_context scope_marker: s64 = 7;

push_copy :: () #expand {
    copied := context;
    `push_context, defer_pop copied;
}

main :: () {
    context.scope_marker = 55;

    {
        push_copy();
        assert(context.scope_marker == 55);
        context.scope_marker = 91;
        assert(context.scope_marker == 91);
    }

    assert(context.scope_marker == 55);
    print("ok\n");
}
