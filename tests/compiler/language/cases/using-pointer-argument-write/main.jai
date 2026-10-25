#import "Basic";

Pool :: struct {
    current: s64;
}

Arena :: struct {
    pool: Pool;
}

Scratch :: struct {
    #as using arena: *Arena;
    start: s64;
}

rewind :: (using scratch: Scratch) {
    pool.current = start;
}

main :: () {
    arena: Arena;
    arena.pool.current = 9;
    rewind(.{ arena = *arena, start = 3 });
    assert(arena.pool.current == 3, "a member behind a pointer the argument holds can be written");
    print("ok\n");
}
