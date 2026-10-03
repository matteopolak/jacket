#add_context marker: int;
set_marker :: (value: int) #expand {
    old_marker := context.marker;
    context.marker = value;
    `defer context.marker = old_marker;
}

main :: () -> int {
    context.marker = 40;
    pushed: #Context;
    pushed.marker = 41;
    push_context pushed {
        {
            set_marker(42);
            if context.marker != 42 return 1;
        }
        if context.marker != 41 return 2;
    }
    return context.marker + 2;
}
