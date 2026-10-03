Allocator :: struct {
    proc: (data:*int)->int #no_context;
    data: *int;
}
Context :: struct { allocator:Allocator; }
context:Context;
read :: (allocator := context.allocator)->int #no_context {
    return allocator.proc(allocator.data);
}
load :: (data:*int)->int #no_context { return data.*; }
main :: ()->int #no_context {
    value := 40;
    context.allocator.proc = load;
    context.allocator.data = *value;
    if read() != 40 return 1;
    value = 42;
    callback := read;
    {
        context:Context;
        // The default belongs to read's file, despite this caller's lexical shadow.
        return callback();
    }
}
