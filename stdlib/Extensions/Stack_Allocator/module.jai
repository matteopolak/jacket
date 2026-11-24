// A last-in, first-out allocator over a fixed block of memory.
//
// Allocation bumps a cursor and records a small header before each block; `release` of the newest block moves the
// cursor back, so scoped data costs no more than a pointer move. `mark` and `rewind` free everything allocated since a
// point in one step. Releasing a block that is not the newest does nothing until a later `rewind` or `reset` covers it.
// When the block is full allocation returns null. It does not lock: use one stack per thread.
//
// ```jai
// #import "Extensions/Stack_Allocator";
//
// stack: Stack_Allocator;
// assert(init(*stack, 16 * 1024));
// defer fini(*stack);
//
// scope := mark(*stack);
// scratch := allocate(*stack, 1000);
// assert(scratch != null);
// rewind(*stack, scope); // scratch is gone
// ```

CAPS_VERSION_STRING :: "Extensions/Stack_Allocator";

// The state of a stack allocator. Keep it at a fixed address while an `Allocator` refers to it. A zeroed one is not ready:
// call `init` or `init_in_memory`.
Stack_Allocator :: struct {
    memory: *u8;
    capacity: s64;
    top: s64; // the first unused byte
    newest: s64 = -1; // where the newest block's payload starts, or -1 when empty
    high_water: s64;
    count: s64; // live blocks, counting those released out of order that a rewind has not yet covered

    owned_memory: *void;
    backing: Allocator;
}

// A point in the stack to `rewind` to.
Stack_Mark :: struct {
    top: s64;
    newest: s64;
    count: s64;
}

// What a stack holds right now.
Stack_Statistics :: struct {
    capacity: s64;
    used: s64; // headers and padding included
    high_water: s64; // the most `used` has reached since `init`
    blocks: s64; // blocks allocated and not yet popped
}

// Takes `capacity` bytes from `backing`, which `fini` gives back. Returns false when the backing allocator fails.
//
// ```jai
// stack: Stack_Allocator;
// assert(init(*stack, 4096));
// defer fini(*stack);
// ```
init :: (stack: *Stack_Allocator, capacity: s64, backing := context.allocator) -> bool {
    assert(stack.memory == null, "stack allocator is already initialized");
    if capacity <= 0 return false;

    memory := backing.proc(.ALLOCATE, capacity, 0, null, backing.data);
    if !memory return false;

    init_in_memory(stack, memory, capacity);
    stack.owned_memory = memory;
    stack.backing = backing;

    return true;
}

// Uses `memory`, which the caller keeps owning, as the stack's block.
//
// ```jai
// storage: [1024] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// assert(allocate(*stack, 100) != null);
// ```
init_in_memory :: (stack: *Stack_Allocator, memory: *void, bytes: s64) {
    assert(stack.memory == null, "stack allocator is already initialized");
    assert(memory != null && bytes > 0, "stack allocator needs memory");
    stack.* = .{ memory = memory, capacity = bytes };
}

// Returns the block taken by `init`. Everything allocated from the stack becomes invalid.
//
// ```jai
// stack: Stack_Allocator;
// assert(init(*stack, 4096));
// fini(*stack);
// ```
fini :: (stack: *Stack_Allocator) {
    if stack.owned_memory stack.backing.proc(.FREE, 0, 0, stack.owned_memory, stack.backing.data);
    stack.* = .{};
}

// Returns `size` bytes aligned to `alignment`, a power of two, or null when the stack is full. The memory is not
// cleared.
//
// ```jai
// storage: [1024] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// block := allocate(*stack, 100, 64);
// assert((cast(u64) block & 63) == 0);
// assert(allocate(*stack, 5000) == null);
// ```
allocate :: (stack: *Stack_Allocator, size: s64, alignment := 16) -> *void {
    assert(alignment > 0 && (alignment & (alignment - 1)) == 0, "alignment must be a power of two");
    if size < 0 return null;

    base := cast(u64) stack.memory;
    mask := cast(u64) alignment - 1;
    payload := ((base + cast(u64) stack.top + cast(u64) HEADER_SIZE + mask) & ~mask) - base;

    if cast(s64) payload > stack.capacity || size > stack.capacity - cast(s64) payload return null;

    header := cast(*Header) (stack.memory + payload - HEADER_SIZE);
    header.previous_top = stack.top;
    header.previous_newest = stack.newest;
    header.size = size;

    stack.top = cast(s64) payload + size;
    stack.newest = cast(s64) payload;
    stack.count += 1;
    stack.high_water = max(stack.high_water, stack.top);

    return stack.memory + payload;
}

// Frees a block. When it is the newest, the cursor moves back over it (and over any blocks released out of order
// beneath it). Otherwise nothing happens yet and the block is freed by the `rewind` or `reset` that covers it. Returns
// whether the cursor moved. Null is ignored.
//
// ```jai
// storage: [1024] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// a := allocate(*stack, 100);
// b := allocate(*stack, 100);
// assert(!release(*stack, a)); // not the newest
// assert(release(*stack, b));
// assert(statistics(*stack).used == 0); // freeing b exposed a, which was already released
// ```
release :: (stack: *Stack_Allocator, memory: *void) -> bool {
    if !memory return false;

    payload := cast(*u8) memory - stack.memory;

    assert(payload >= HEADER_SIZE && payload <= stack.top, "block does not belong to this stack allocator");
    header := header_at(stack, payload);

    if payload != stack.newest {
        header.size = RELEASED; // remember that it is dead for when the blocks above it pop
        return false;
    }

    pop_newest(stack);

    // Blocks released earlier, out of order, now sit on top.
    while stack.newest >= 0 && header_at(stack, stack.newest).size == RELEASED pop_newest(stack);

    return true;
}

// Notes the current position so `rewind` can return to it.
//
// ```jai
// storage: [1024] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// start := mark(*stack);
// allocate(*stack, 100);
// rewind(*stack, start);
// ```
mark :: (stack: *Stack_Allocator) -> Stack_Mark {
    return .{ stack.top, stack.newest, stack.count };
}

// Frees every block allocated after `point`, whether or not it was released. Rewind to marks in the reverse of the order
// they were taken.
//
// ```jai
// storage: [1024] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// keep := allocate(*stack, 100);
// start := mark(*stack);
// allocate(*stack, 200);
// allocate(*stack, 300);
// rewind(*stack, start);
// assert(statistics(*stack).blocks == 1);
// ```
rewind :: (stack: *Stack_Allocator, point: Stack_Mark) {
    assert(point.top <= stack.top, "cannot rewind to a point that is already freed");
    stack.top = point.top;
    stack.newest = point.newest;
    stack.count = point.count;
}

// Frees everything.
//
// ```jai
// storage: [256] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// allocate(*stack, 100);
// reset(*stack);
// assert(statistics(*stack).used == 0);
// ```
reset :: (stack: *Stack_Allocator) {
    stack.top = 0;
    stack.newest = -1;
    stack.count = 0;
}

// Changes a block's size, keeping its contents up to the smaller size. The newest block grows or shrinks in place; an
// older block moves to the top, and its old space is freed when the blocks above it pop. Returns null, leaving the old
// block valid, when there is no room. A null `memory` allocates.
//
// ```jai
// storage: [1024] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// block := allocate(*stack, 10);
// assert(resize(*stack, block, 400) == block);
// ```
resize :: (stack: *Stack_Allocator, memory: *void, new_size: s64) -> *void {
    if !memory return allocate(stack, new_size);
    if new_size < 0 return null;

    payload := cast(*u8) memory - stack.memory;

    assert(payload >= HEADER_SIZE && payload <= stack.top, "block does not belong to this stack allocator");
    header := header_at(stack, payload);

    if payload == stack.newest {
        if new_size > stack.capacity - payload return null;

        header.size = new_size;
        stack.top = payload + new_size;
        stack.high_water = max(stack.high_water, stack.top);

        return memory;
    }

    moved := allocate(stack, new_size);
    if !moved return null;

    memcpy(moved, memory, min(header.size, new_size));
    release(stack, memory);

    return moved;
}

// Whether `memory` lies in the part of the stack in use.
//
// ```jai
// storage: [256] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// assert(owns(*stack, allocate(*stack, 8)));
// ```
owns :: (stack: *Stack_Allocator, memory: *void) -> bool {
    address := cast(*u8) memory;
    return stack.memory != null && address >= stack.memory + HEADER_SIZE && address < stack.memory + stack.top;
}

// The stack's capacity and use.
//
// ```jai
// storage: [256] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// allocate(*stack, 100);
// stats := statistics(*stack);
// assert(stats.blocks == 1 && stats.used >= 100);
// ```
statistics :: (stack: *Stack_Allocator) -> Stack_Statistics {
    return .{ stack.capacity, stack.top, stack.high_water, stack.count };
}

// The stack as an allocator for `context.allocator`.
//
// ```jai
// storage: [1024] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// context.allocator = as_allocator(*stack);
// numbers := NewArray(10, int);
// assert(numbers.count == 10);
// ```
as_allocator :: (stack: *Stack_Allocator) -> Allocator {
    return .{ stack_allocator_proc, stack };
}

// The `Allocator_Proc` of a `Stack_Allocator`. A full stack returns null. Free pops the newest block and is otherwise
// deferred (see `release`).
//
// ```jai
// storage: [256] u8;
// stack: Stack_Allocator;
// init_in_memory(*stack, storage.data, storage.count);
// block := stack_allocator_proc(.ALLOCATE, 32, 0, null, *stack);
// stack_allocator_proc(.FREE, 0, 0, block, *stack);
// ```
stack_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = CAPS_VERSION_STRING;
        }

        caps := Allocator_Caps.FREE | .ACTUALLY_RESIZE | .IS_THIS_YOURS | .HINT_I_AM_A_FAST_BUMP_ALLOCATOR;

        return cast(*void) cast(u64) caps;
    }

    stack := cast(*Stack_Allocator) allocator_data;

    assert(stack != null, "the stack allocator needs its stack as data");

    if mode == .ALLOCATE return allocate(stack, requested_size);
    if mode == .RESIZE return resize(stack, old_memory, requested_size);

    if mode == .FREE {
        release(stack, old_memory);
        return null;
    }

    if mode == .IS_THIS_YOURS {
        if old_memory && owns(stack, old_memory) return cast(*void) 1;
        return null;
    }

    return null;
}

#scope_file

#import "Basic";

// Sits just before each block's payload.
Header :: struct {
    previous_top: s64;
    previous_newest: s64;
    size: s64; // RELEASED once the block was freed out of order
}

HEADER_SIZE :: 24;
RELEASED :: -1;

header_at :: (stack: *Stack_Allocator, payload: s64) -> *Header {
    return cast(*Header) (stack.memory + payload - HEADER_SIZE);
}

pop_newest :: (stack: *Stack_Allocator) {
    header := header_at(stack, stack.newest);
    stack.top = header.previous_top;
    stack.newest = header.previous_newest;
    stack.count -= 1;
}
