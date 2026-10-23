// `#bake_arguments f(,, allocator = a)` names a context field, not a parameter: the copy runs
// with that field set, like a call written with `,,`. Parameters can be baked alongside.
#import "Basic";

marked_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    return null;
}

marked :: Allocator.{ marked_allocator, null };

uses_marked :: () -> bool {
    return context.allocator.proc == marked_allocator;
}

bonus :: (n: int, extra: int) -> int {
    if uses_marked() return n + extra;
    return n;
}

with_marked :: #bake_arguments uses_marked(,, allocator = marked);
bonus_marked :: #bake_arguments bonus(extra = 10,, allocator = marked);

main :: () {
    assert(!uses_marked());
    assert(with_marked());
    assert(!uses_marked());

    assert(bonus_marked(1) == 11);
    assert(bonus(1, 10) == 1);

    print("ok\n");
}
