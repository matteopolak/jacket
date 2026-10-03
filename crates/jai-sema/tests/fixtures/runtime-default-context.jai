Allocator :: struct { marker:int; }
#add_context allocator:Allocator;
read :: (allocator:Allocator=context.allocator)->int { return allocator.marker; }
main :: ()->int {
    context.allocator.marker = 40;
    if read() != 40 return 1;
    different := context;
    different.allocator.marker = 42;
    answer := 0;
    push_context different { answer = read(); }
    if context.allocator.marker != 40 return 2;
    return answer;
}
