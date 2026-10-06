#import "Basic";
my_alloc :: (mode: Allocator_Mode, size: s64, old: s64, p: *void, data: *void) -> *void { return alloc(size); }
main :: () { context.allocator = .{my_alloc, null}; p := alloc(10); }
