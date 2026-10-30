allocator :: Allocator.{ allocator_proc, null };
allocator_default_heap: *Heap;

initialize_if_needed :: inline () -> did_init: bool {
    acquire();
    defer unlock();
    return initialize_locked();
}

is_initialized :: inline () -> bool {
    acquire();
    defer unlock();
    return initialized;
}

finalize :: () {
    acquire();
    defer unlock();

    while heaps {
        state := heaps;
        heaps = state.next;
        clear_locked(state);
        c_free(state);
    }

    allocator_default_heap = null;
    initialized = false;

    if live.data c_free(live.data);

    live.data = null;
    live.count = 0;
    live_count = 0;
}

// A fork child owns the inherited state exclusively; do not touch live blocks.
unsafe_reset :: inline () {
    guard = 0;
}

usable_size :: inline (ptr: *void) -> s64 {
    if !ptr return 0;

    acquire();
    defer unlock();
    for_heap := heaps;
    while for_heap {
        block := find_block_locked(for_heap, ptr);
        if block return block.size;
        for_heap = for_heap.next;
    }

    argument_error("Default_Allocator.usable_size requires a live owned pointer");

    return 0;
}

is_this_yours :: (heap: *Heap, memory: *void) -> bool {
    acquire();
    defer unlock();
    state := find_heap_locked(heap);
    return state != null && find_block_locked(state, memory) != null;
}

heap_free_all :: (heap: *Heap) {
    acquire();
    defer unlock();
    state := find_heap_locked(heap);
    if !state {
        argument_error("Default_Allocator.heap_free_all requires a live heap");
        return;
    }

    clear_locked(state);
}

global_statistics :: () -> Global_Statistics {
    // The standard variant explicitly returns dummy statistics.
    return .{};
}

heap_statistics :: (heap: *Heap) -> Heap_Statistics {
    acquire();
    defer unlock();
    if !find_heap_locked(heap) argument_error("Default_Allocator.heap_statistics requires a live heap");

    // No freed blocks or spans are cached by this independent backend.
    return .{};
}

allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = "Default_Allocator";
        }

        return cast(*void) cast(u64) (Allocator_Caps.MULTIPLE_THREADS | .CREATE_HEAP | .FREE | .ACTUALLY_RESIZE | .IS_THIS_YOURS | .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR);
    }

    if mode == .SHUTDOWN {
        finalize();
        return null;
    }

    if mode == .THREAD_START || mode == .THREAD_STOP return null;

    acquire();
    defer unlock();

    if mode == .STARTUP {
        initialize_locked();
        return null;
    }

    if mode == .CREATE_HEAP return create_heap_locked();

    if !initialized {
        initialize_locked();
        if !initialized return null;
    }

    heap := cast(*Heap) allocator_data;
    if !heap heap = allocator_default_heap;

    state := find_heap_locked(heap);
    if !state {
        argument_error("Default_Allocator received an invalid heap handle");
        return null;
    }

    if mode == .IS_THIS_YOURS return cast(*void) cast(u64) (find_block_locked(state, old_memory) != null);

    if mode == .DESTROY_HEAP {
        remove_heap_locked(state);
        return null;
    }

    if mode == .ALLOCATE {
        if !valid_size(requested_size) return null;
        return allocate_locked(state, requested_size);
    }

    if mode == .FREE || mode == .RESIZE {
        if mode == .RESIZE && !valid_size(requested_size) return null;

        if !old_memory {
            if mode == .RESIZE return allocate_locked(state, requested_size);
            return null;
        }

        block := find_block_locked(state, old_memory);
        if !block || old_size < 0 || old_size > block.size {
            argument_error("Default_Allocator received an invalid pointer or source extent");
            return null;
        }

        if mode == .FREE {
            unlink_block_locked(state, block);
            live_remove(old_memory);
            release_contents(old_memory);
            return null;
        }

        before := block.previous;
        after := block.next;
        contents := regrow_contents(old_memory, requested_size, size_of(Allocation_Header));

        if !contents return null;

        // The block may have moved; the entry removed here leaves room for the one added.
        live_remove(old_memory);
        live_insert(contents, state);

        // The block's bookkeeping stays where it was; rebuild it around the moved contents.
        resized := cast(*Allocation_Header) (cast(*u8) contents - size_of(Allocation_Header));
        resized.previous = before;
        resized.next = after;
        resized.size = requested_size;
        if before before.next = resized;
        else state.first = resized;
        if after after.previous = resized;

        return contents;
    }

    argument_error("unsupported Default_Allocator mode");

    return null;
}

#scope_file
using #import "Basic";

Heap_State :: struct {
    public_heap: Heap;
    first: *Allocation_Header;
    next: *Heap_State;
}

// Sits right before the block's contents; `region` must stay last (see aligned-blocks.jai).
Allocation_Header :: struct #align 16 {
    previous: *Allocation_Header;
    next: *Allocation_Header;
    size: s64;
    region: *void;
}

heaps: *Heap_State;
initialized: bool;
guard: s32;
next_id: s32;

#import "internal/libc";

acquire :: () {
    acquired := compare_and_swap(*guard, cast(s32) 0, cast(s32) 1);
    while !acquired acquired = compare_and_swap(*guard, cast(s32) 0, cast(s32) 1);
}

unlock :: () {
    released := compare_and_swap(*guard, cast(s32) 1, cast(s32) 0);
    if !released argument_error("Default_Allocator lock ownership lost");
}

argument_error :: (message: string) {
    if ENABLE_ASSERTS assert(false, message);
}

valid_size :: (size: s64) -> bool {
    if size < 0 || size > 0x7fffffffffffffff - size_of(Allocation_Header) return false;
    if ENABLE_VALIDATE_ARGS && size >= 0x7fffffffffffffff - 65536 return false;
    return true;
}

initialize_locked :: () -> bool {
    if initialized return false;

    heap := create_heap_locked();
    if !heap return false;

    allocator_default_heap = heap;
    initialized = true;

    return true;
}

create_heap_locked :: () -> *Heap {
    state := cast(*Heap_State) c_malloc(size_of(Heap_State));
    if !state return null;

    state.* = .{};
    state.next = heaps;
    next_id += 1;
    state.public_heap.id = next_id;
    state.public_heap.lock = *guard;
    if heaps state.public_heap.next_heap = *heaps.public_heap;

    heaps = state;

    return *state.public_heap;
}

find_heap_locked :: (heap: *Heap) -> *Heap_State {
    state := heaps;
    while state {
        if *state.public_heap == heap return state;
        state = state.next;
    }

    return null;
}

find_block_locked :: (state: *Heap_State, memory: *void) -> *Allocation_Header {
    if !memory return null;

    at := live_find(memory);
    if at < 0 || live[at].owner != state return null;

    return cast(*Allocation_Header) (cast(*u8) memory - size_of(Allocation_Header));
}

// Every live block's contents address, with the heap that owns it, in an open-addressing table
// (linear probing, backward-shift removal), so telling whether a pointer is a live block of a
// heap takes a lookup rather than a walk down the heap's list. The table doubles when it is half
// full and is only touched under the allocator's lock.
Live_Entry :: struct {
    memory: *void;
    owner: *Heap_State;
}

live: [] Live_Entry;
live_count: s64;

// Knuth's multiplicative constant for 32-bit keys.
SLOT_MULTIPLIER :: 2654435761;

live_slot :: (memory: *void, mask: s64) -> s64 {
    address := (cast(u64) memory) >> 4;
    folded := (address ^ (address >> 32)) & 0xffffffff;
    return cast(s64) (((folded * SLOT_MULTIPLIER) >> 12) & cast(u64) mask);
}

live_find :: (memory: *void) -> s64 {
    if live.count == 0 return -1;

    mask := live.count - 1;
    at := live_slot(memory, mask);

    while live[at].memory {
        if live[at].memory == memory return at;
        at = (at + 1) & mask;
    }

    return -1;
}

live_grow :: () -> bool {
    capacity := ifx live.count == 0 then cast(s64) 1024 else live.count * 2;

    fresh: [] Live_Entry;
    fresh.data = c_malloc(cast(u64) (capacity * size_of(Live_Entry)));
    if !fresh.data return false;

    fresh.count = capacity;

    index := 0;
    while index < capacity {
        fresh[index] = .{};
        index += 1;
    }

    mask := capacity - 1;

    index = 0;
    while index < live.count {
        entry := live[index];
        if entry.memory {
            at := live_slot(entry.memory, mask);
            while fresh[at].memory at = (at + 1) & mask;
            fresh[at] = entry;
        }

        index += 1;
    }

    if live.data c_free(live.data);

    live = fresh;

    return true;
}

live_insert :: (memory: *void, owner: *Heap_State) -> bool {
    if (live_count + 1) * 2 > live.count && !live_grow() return false;

    mask := live.count - 1;
    at := live_slot(memory, mask);

    while live[at].memory at = (at + 1) & mask;

    live[at] = .{ memory, owner };
    live_count += 1;

    return true;
}

live_remove :: (memory: *void) {
    at := live_find(memory);
    if at < 0 return;

    mask := live.count - 1;
    hole := at;
    next := (at + 1) & mask;

    while live[next].memory {
        // An entry may fill the hole unless its probe sequence starts after it.
        home := live_slot(live[next].memory, mask);
        if ((next - home) & mask) >= ((next - hole) & mask) {
            live[hole] = live[next];
            hole = next;
        }

        next = (next + 1) & mask;
    }

    live[hole] = .{};
    live_count -= 1;
}

allocate_locked :: (state: *Heap_State, size: s64) -> *void {
    contents := obtain_contents(size, size_of(Allocation_Header));
    if !contents return null;

    if !live_insert(contents, state) {
        release_contents(contents);
        return null;
    }

    block := cast(*Allocation_Header) (cast(*u8) contents - size_of(Allocation_Header));
    block.previous = null;
    block.next = state.first;
    block.size = size;
    if state.first state.first.previous = block;

    state.first = block;

    return contents;
}

unlink_block_locked :: (state: *Heap_State, block: *Allocation_Header) {
    if block.previous block.previous.next = block.next;
    else state.first = block.next;
    if block.next block.next.previous = block.previous;
}

clear_locked :: (state: *Heap_State) {
    while state.first {
        block := state.first;

        state.first = block.next;
        contents := cast(*u8) block + size_of(Allocation_Header);

        live_remove(contents);
        release_contents(contents);
    }
}

remove_heap_locked :: (state: *Heap_State) {
    clear_locked(state);
    previous: *Heap_State;
    cursor := heaps;

    while cursor != state {
        previous = cursor;
        cursor = cursor.next;
    }

    if previous {
        previous.next = state.next;
        previous.public_heap.next_heap = state.public_heap.next_heap;
    } else heaps = state.next;

    if allocator_default_heap == *state.public_heap {
        allocator_default_heap = null;
        initialized = false;
    }

    c_free(state);
}
