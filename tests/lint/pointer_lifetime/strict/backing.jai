#import "Basic";

Arena :: struct {
    base: *void;
}

make :: () -> *void {
    return alloc(128);
}

release :: (p: *void) {} @FreesArgument
read :: (p: *u8) -> u8 {
    return p.*;
}

borrow_arena :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    area := cast(*Arena) data;
    return area.base;
}

independent :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    return make();
}

backing_released :: () {
    backing := make();
    arena: Arena;
    arena.base = backing;
    a: Allocator;
    a.proc = borrow_arena;
    a.data = *arena;
    p := a.proc(.ALLOCATE, 8, 0, null, a.data);

    release(backing);
    read(cast(*u8) p);
}

backing_released_before_allocator_definition :: () {
    backing := make();
    arena: Arena;
    arena.base = backing;
    a: Allocator;
    a.proc = late_allocator;
    a.data = *arena;
    p := a.proc(.ALLOCATE, 8, 0, null, a.data);

    release(backing);
    read(cast(*u8) p);
}

metadata_is_not_backing :: () {
    metadata := make();
    arena: Arena;
    arena.base = metadata;
    a: Allocator;
    a.proc = independent;
    a.data = *arena;
    p := a.proc(.ALLOCATE, 8, 0, null, a.data);

    release(metadata);
    read(cast(*u8) p);
}

late_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    area := cast(*Arena) data;
    return area.base;
}
