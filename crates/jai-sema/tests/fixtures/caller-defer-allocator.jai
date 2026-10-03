Allocator :: #type(value: int) -> int #no_context;
first :: (value: int) -> int #no_context { return value + 1; }
second :: (value: int) -> int #no_context { return value + 2; }
#add_context allocator: Allocator;

push_allocator :: (allocator: Allocator) #expand #no_debug {
    old_allocator := context.allocator;
    context.allocator = allocator;
    `defer context.allocator = old_allocator;
}

main :: () -> int {
    context.allocator = first;
    old_allocator := second;
    {
        push_allocator(second);
        if context.allocator(40) != 42 return 1;
        if old_allocator(40) != 42 return 2;
    }
    return context.allocator(41);
}
