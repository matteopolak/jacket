#import "Basic";

USE_ARENA :: true;

main :: () {
    arena := context.allocator;

    // Scoped by its own block inside the arm.
    #if USE_ARENA {
        {
            push_allocator(arena);
            print("arena\n");
        }
    }

    // Outside any `#if`.
    push_allocator(arena);

    // The arm holds no push.
    #if USE_ARENA {
        print("%\n", USE_ARENA);
    }
}

scoped :: (arena: Allocator) {
    // A run-time `if` has a scope of its own.
    if arena.proc {
        push_allocator(arena);
        print("arena\n");
    }
}
