#import "Basic";

USE_ARENA :: true;
USE_POOL :: false;

main :: () {
    arena := context.allocator;

    #if USE_ARENA {
        push_allocator(arena);
    }

    #if USE_POOL {
        print("pool\n");
    } else {
        push_allocator(arena);
    }

    #if USE_ARENA push_allocator(arena.proc, arena.data);

    print("%\n", USE_ARENA);
}
