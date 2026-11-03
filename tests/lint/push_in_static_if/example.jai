#import "Basic";

USE_ARENA :: true;

main :: () {
    arena := context.allocator;

    #if USE_ARENA {
        push_allocator(arena);
    }

    print("%\n", alloc(8) != null);
}
