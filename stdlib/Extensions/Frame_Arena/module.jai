// Arenas for per-frame data: a fixed block of memory that allocation walks through and `reset` rewinds.
//
// A `Frame_Arena` suits data that lives for one frame of a loop (a game frame, a request, a render pass): allocate as
// freely as needed, then `reset` and everything is free again. A `Double_Frame_Arena` pairs two of them and swaps on
// `next_frame`, so what the previous frame built stays valid for the whole current frame. Allocation is a pointer bump;
// when the block is full it returns null. Neither arena locks: use one per thread.
//
// `Pool` and `Flat_Pool` also bump-allocate, but a pool grows without limit or asserts when its address range runs out.
// An arena has a fixed capacity, reports its high-water mark so you can size it, and never touches the system after setup.
//
// ```jai
// #import "Extensions/Frame_Arena";
//
// arena: Frame_Arena;
// assert(init(*arena, 64 * 1024));
// defer fini(*arena);
//
// for frame: 0..2 {
//     context.allocator = as_allocator(*arena);
//     names: [..] string;
//     array_add(*names, "scratch");
//     assert(names.count == 1);
//     reset(*arena); // frees everything this frame allocated
// }
// ```

CAPS_VERSION_STRING :: "Extensions/Frame_Arena";

// A fixed block of memory handed out by bumping a cursor. Keep it at a fixed address while an `Allocator` refers to it.
// A zeroed arena is not ready: call `init` or `init_in_memory`.
Frame_Arena :: struct {
    // Fills memory with `0xcc` on `reset`, so a stale pointer from an earlier frame shows up as garbage.
    poison_on_reset: bool;

    memory: *u8;
    capacity: s64;
    used: s64;
    last_start: s64 = -1; // where the latest allocation begins, so it can grow in place
    high_water: s64;
    frame: s64; // how many times the arena has been reset

    owned_memory: *void;
    backing: Allocator;
}

// What an arena holds right now.
Frame_Arena_Statistics :: struct {
    capacity: s64;
    used: s64;
    high_water: s64; // the most `used` has reached since `init`, across resets
    frame: s64;
}

// Takes `capacity` bytes from `backing`, which `fini` gives back. Returns false when the backing allocator fails.
//
// ```jai
// arena: Frame_Arena;
// assert(init(*arena, 4096));
// defer fini(*arena);
// ```
init :: (arena: *Frame_Arena, capacity: s64, backing := context.allocator) -> bool {
    assert(arena.memory == null, "frame arena is already initialized");
    if capacity <= 0 return false;

    memory := backing.proc(.ALLOCATE, capacity, 0, null, backing.data);
    if !memory return false;

    init_in_memory(arena, memory, capacity);
    arena.owned_memory = memory;
    arena.backing = backing;

    return true;
}

// Uses `memory`, which the caller keeps owning, as the arena's block.
//
// ```jai
// storage: [1024] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// assert(allocate(*arena, 100) != null);
// ```
init_in_memory :: (arena: *Frame_Arena, memory: *void, bytes: s64) {
    assert(arena.memory == null, "frame arena is already initialized");
    assert(memory != null && bytes > 0, "frame arena needs memory");
    arena.* = .{ memory = memory, capacity = bytes };
}

// Returns the block taken by `init`. Everything allocated from the arena becomes invalid.
//
// ```jai
// arena: Frame_Arena;
// assert(init(*arena, 4096));
// fini(*arena);
// ```
fini :: (arena: *Frame_Arena) {
    if arena.owned_memory arena.backing.proc(.FREE, 0, 0, arena.owned_memory, arena.backing.data);
    arena.* = .{};
}

// Returns `size` bytes aligned to `alignment`, a power of two, or null when the arena is full. The memory is not cleared.
//
// ```jai
// storage: [256] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// block := allocate(*arena, 100, 64);
// assert((cast(u64) block & 63) == 0);
// assert(allocate(*arena, 1000) == null);
// ```
allocate :: (arena: *Frame_Arena, size: s64, alignment := 16) -> *void {
    assert(alignment > 0 && (alignment & (alignment - 1)) == 0, "alignment must be a power of two");
    if size < 0 return null;

    base := cast(u64) arena.memory;
    mask := cast(u64) alignment - 1;
    start := ((base + cast(u64) arena.used + mask) & ~mask) - base;

    if cast(s64) start > arena.capacity || size > arena.capacity - cast(s64) start return null;

    arena.last_start = cast(s64) start;
    arena.used = cast(s64) start + size;
    arena.high_water = max(arena.high_water, arena.used);

    return arena.memory + start;
}

// Changes a block's size, keeping its contents up to the smaller size. The most recent allocation grows or shrinks in
// place; any other block moves to the end of the arena and its old space stays used until `reset`. Returns null, leaving
// the old block valid, when there is no room. A null `memory` allocates.
//
// ```jai
// storage: [1024] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// block := allocate(*arena, 10);
// assert(resize(*arena, block, 10, 500) == block);
// ```
resize :: (arena: *Frame_Arena, memory: *void, old_size: s64, new_size: s64) -> *void {
    if !memory return allocate(arena, new_size);
    if new_size < 0 return null;

    offset := cast(*u8) memory - arena.memory;
    if offset == arena.last_start {
        if new_size > arena.capacity - offset return null;

        arena.used = offset + new_size;
        arena.high_water = max(arena.high_water, arena.used);
        return memory;
    }

    moved := allocate(arena, new_size);
    if moved memcpy(moved, memory, min(old_size, new_size));

    return moved;
}

// Starts a new frame: every allocation becomes free and its memory is reused from the start.
//
// ```jai
// storage: [256] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// first := allocate(*arena, 64);
// reset(*arena);
// assert(allocate(*arena, 64) == first);
// ```
reset :: (arena: *Frame_Arena) {
    if arena.poison_on_reset && arena.memory memset(arena.memory, 0xcc, arena.used);

    arena.used = 0;
    arena.last_start = -1;
    arena.frame += 1;
}

// Whether `memory` lies in the part of the arena allocated so far.
//
// ```jai
// storage: [256] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// assert(owns(*arena, allocate(*arena, 8)));
// ```
owns :: (arena: *Frame_Arena, memory: *void) -> bool {
    address := cast(*u8) memory;
    return arena.memory != null && address >= arena.memory && address < arena.memory + arena.used;
}

// The arena's capacity, use and frame count.
//
// ```jai
// storage: [256] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// allocate(*arena, 100);
// reset(*arena);
// stats := statistics(*arena);
// assert(stats.used == 0 && stats.high_water >= 100 && stats.frame == 1);
// ```
statistics :: (arena: *Frame_Arena) -> Frame_Arena_Statistics {
    return .{ arena.capacity, arena.used, arena.high_water, arena.frame };
}

// The arena as an allocator for `context.allocator`.
//
// ```jai
// storage: [1024] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// context.allocator = as_allocator(*arena);
// numbers := NewArray(10, int);
// assert(numbers.count == 10);
// ```
as_allocator :: (arena: *Frame_Arena) -> Allocator {
    return .{ frame_arena_proc, arena };
}

// The `Allocator_Proc` of a `Frame_Arena`. Free does nothing; a full arena returns null.
//
// ```jai
// storage: [256] u8;
// arena: Frame_Arena;
// init_in_memory(*arena, storage.data, storage.count);
// block := frame_arena_proc(.ALLOCATE, 32, 0, null, *arena);
// assert(block != null);
// ```
frame_arena_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    arena := cast(*Frame_Arena) allocator_data;

    assert(arena != null, "the frame arena allocator needs its arena as data");

    if mode == .ALLOCATE return allocate(arena, requested_size);
    if mode == .RESIZE return resize(arena, old_memory, old_size, requested_size);

    if mode == .IS_THIS_YOURS {
        if old_memory && owns(arena, old_memory) return cast(*void) 1;
        return null;
    }

    if mode == .CAPS return caps_result(old_memory);

    return null; // free and the lifecycle modes do nothing
}

// Two arenas used in turns: allocate in the current one, and read what the previous frame left in the other.
//
// `next_frame` makes the previous arena current again after emptying it, so data lives for exactly two frames: the one
// that allocated it and the one after.
Double_Frame_Arena :: struct {
    arenas: [2] Frame_Arena;
    current_index: s32;
}

// Sets up two arenas of `capacity` bytes each from `backing`. Returns false when the backing allocator fails.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 4096));
// defer fini(*frames);
// ```
init :: (frames: *Double_Frame_Arena, capacity: s64, backing := context.allocator) -> bool {
    if !init(*frames.arenas[0], capacity, backing) return false;

    if !init(*frames.arenas[1], capacity, backing) {
        fini(*frames.arenas[0]);
        return false;
    }

    frames.current_index = 0;

    return true;
}

// Returns both blocks.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 4096));
// fini(*frames);
// ```
fini :: (frames: *Double_Frame_Arena) {
    fini(*frames.arenas[0]);
    fini(*frames.arenas[1]);
    frames.current_index = 0;
}

// The arena new allocations come from.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 1024));
// defer fini(*frames);
// assert(current(*frames) != previous(*frames));
// ```
current :: (frames: *Double_Frame_Arena) -> *Frame_Arena {
    return *frames.arenas[frames.current_index];
}

// The arena of the frame before, still holding that frame's data.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 1024));
// defer fini(*frames);
// block := allocate(*frames, 8);
// next_frame(*frames);
// assert(owns(previous(*frames), block));
// ```
previous :: (frames: *Double_Frame_Arena) -> *Frame_Arena {
    return *frames.arenas[1 - frames.current_index];
}

// Allocates from the current arena.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 1024));
// defer fini(*frames);
// assert(allocate(*frames, 100) != null);
// ```
allocate :: (frames: *Double_Frame_Arena, size: s64, alignment := 16) -> *void {
    return allocate(current(frames), size, alignment);
}

// Ends the frame: the arena of the frame before is emptied and becomes current, and this frame's data becomes the
// previous frame's. Pointers from two frames ago are invalid after the call.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 1024));
// defer fini(*frames);
// mine := cast(*int) allocate(*frames, size_of(int));
// mine.* = 42;
// next_frame(*frames);
// assert(mine.* == 42); // still readable during the next frame
// next_frame(*frames); // now it is reused
// ```
next_frame :: (frames: *Double_Frame_Arena) {
    frames.current_index = 1 - frames.current_index;
    reset(current(frames));
}

// Whether `memory` lies in either arena.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 1024));
// defer fini(*frames);
// assert(owns(*frames, allocate(*frames, 8)));
// ```
owns :: (frames: *Double_Frame_Arena, memory: *void) -> bool {
    return owns(*frames.arenas[0], memory) || owns(*frames.arenas[1], memory);
}

// The pair as an allocator for `context.allocator`; it always allocates from the current arena, so it keeps working
// across `next_frame`.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 1024));
// defer fini(*frames);
// context.allocator = as_allocator(*frames);
// text := copy_string("frame data");
// next_frame(*frames);
// assert(text == "frame data");
// ```
as_allocator :: (frames: *Double_Frame_Arena) -> Allocator {
    return .{ double_frame_arena_proc, frames };
}

// The `Allocator_Proc` of a `Double_Frame_Arena`. Free does nothing; a full arena returns null.
//
// ```jai
// frames: Double_Frame_Arena;
// assert(init(*frames, 1024));
// defer fini(*frames);
// block := double_frame_arena_proc(.ALLOCATE, 16, 0, null, *frames);
// assert(block != null);
// ```
double_frame_arena_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    frames := cast(*Double_Frame_Arena) allocator_data;

    assert(frames != null, "the double frame allocator needs its arenas as data");

    if mode == .ALLOCATE return allocate(current(frames), requested_size);

    if mode == .RESIZE {
        // A block from the previous frame is copied into the current one instead of being resized.
        if old_memory && owns(previous(frames), old_memory) {
            moved := allocate(current(frames), requested_size);
            if moved memcpy(moved, old_memory, min(old_size, requested_size));
            return moved;
        }

        return resize(current(frames), old_memory, old_size, requested_size);
    }

    if mode == .IS_THIS_YOURS {
        if old_memory && owns(frames, old_memory) return cast(*void) 1;
        return null;
    }

    if mode == .CAPS return caps_result(old_memory);

    return null;
}

#scope_file

#import "Basic";

caps_result :: (name_out: *void) -> *void {
    if name_out {
        (cast(*string) name_out).* = CAPS_VERSION_STRING;
    }

    caps := Allocator_Caps.ACTUALLY_RESIZE | .IS_THIS_YOURS | .HINT_I_AM_A_FAST_BUMP_ALLOCATOR | .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE;

    return cast(*void) cast(u64) caps;
}
