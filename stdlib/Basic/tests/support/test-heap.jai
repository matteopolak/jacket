#import "Basic";

heap_destroyed: bool;
heap_blocks_freed: int;

Heap_State :: struct {
    blocks: *Heap_Block;
    underlying: Allocator;
}

Heap_Block :: struct {
    next: *Heap_Block;
    size: s64;
}

heap_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .CREATE_HEAP {
        allocator := context.default_allocator;
        state := cast(*Heap_State) allocator.proc(.ALLOCATE, size_of(Heap_State), 0, null, allocator.data);

        assert(state != null);
        state.* = .{ underlying = allocator };

        return state;
    }

    state := cast(*Heap_State) allocator_data;
    allocator := state.underlying;

    if mode == .CAPS return cast(*void) cast(u64) (Allocator_Caps.CREATE_HEAP | .FREE | .ACTUALLY_RESIZE);

    if mode == .ALLOCATE || mode == .RESIZE {
        block := cast(*Heap_Block) allocator.proc(.ALLOCATE, 32 + size, 0, null, allocator.data);
        if !block return null;

        block.* = .{ next = state.blocks, size = size };
        state.blocks = block;
        result := cast(*u8) block + 32;

        if mode == .RESIZE && old_memory {
            memcpy(result, old_memory, min(size, old_size));
            heap_allocator(.FREE, 0, old_size, old_memory, state);
        }

        return result;
    }

    if mode == .FREE {
        link := *state.blocks;
        while link.* {
            block := link.*;
            if cast(*u8) block + 32 == old_memory {
                link.* = block.next;
                allocator.proc(.FREE, 0, 32 + block.size, block, allocator.data);
                heap_blocks_freed += 1;
                return null;
            }

            link = *block.next;
        }

        assert(false, "unknown heap allocation");

        return null;
    }

    if mode == .DESTROY_HEAP {
        while state.blocks heap_allocator(.FREE, 0, 0, cast(*u8) state.blocks + 32, state);
        allocator.proc(.FREE, 0, size_of(Heap_State), state, allocator.data);
        heap_destroyed = true;
        return null;
    }

    assert(false, "unexpected allocator operation");

    return null;
}
