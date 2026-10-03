Allocator :: struct { marker:int; }
#add_context allocator:Allocator;
read :: (allocator:Allocator=context.allocator)->int { return allocator.marker; }
main :: ()->int {
    context.allocator.marker = 7;
    replacement:Allocator;
    replacement.marker = 42;
    callback := read;
    answer := callback(,,allocator=replacement);
    if context.allocator.marker != 7 return 1;
    return answer;
}
