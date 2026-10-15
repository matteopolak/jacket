#import "Basic";
S :: #import "String";
D :: #import "Basic"(MEMORY_DEBUGGER = true);
Quiet :: #import "Basic"(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);

messages: int;
errors: int;
received: string;

logger :: (message: string, data: *void, info: Log_Info) {
    assert(info.common_flags == .ERROR);
    messages += 1;
    received = copy_string(message);
}

failure :: (location: Source_Code_Location, message: string) -> bool {
    errors += 1;
    return false;
}

main :: () {
    push_context {
        context.logger = logger;
        memory := D.alloc(24);

        push_context {
            context.assertion_failed = failure;
            D._check_alloc(context.allocator, memory, 24);
            wrong := context.allocator;
            wrong.data = cast(*void) 1;
            D._check_free(wrong, memory);
        }

        D.free(memory);
        D.memory_visualizer_per_frame_update();
        plain := Quiet.alloc(8);

        Quiet.free(plain);
        Quiet.memory_visualizer_per_frame_update();
    }

    defer free(received);
    assert(errors == 2);
    assert(messages == 1);
    assert(S.contains(received, "visualization"));

    push_context {
        context.logger = logger;
        heap := D.create_heap(.{ heap_allocator, null });

        push_context {
            context.allocator = heap;
            first_block := cast(*u8) D.alloc(12);
            second_block := cast(*u8) D.alloc(20);

            first_block[0] = 1;
            second_block[0] = 2;
        }

        D.destroy_heap(heap);
        D.memory_visualizer_per_frame_update();
    }

    assert(heap_destroyed && heap_blocks_freed == 2);
    assert(messages == 1);
    bytes: [16] u8;
    pool := Reuse_Pool.{ block = bytes.data };

    push_context {
        context.allocator = .{ reuse_allocator, *pool };
        first := D.alloc(8);

        D.free(first);
        external := alloc(8);

        assert(external == first);
        D.free(external);
        assert(!pool.active);
    }

    print("ok\n");
}

#load "support/test-heap.jai";

Reuse_Pool :: struct {
    block: *u8;
    active: bool;
}

reuse_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    pool := cast(*Reuse_Pool) allocator_data;

    if mode == .CAPS return cast(*void) cast(u64) Allocator_Caps.FREE;

    if mode == .ALLOCATE {
        assert(!pool.active && size <= 16);
        pool.active = true;
        return pool.block;
    }

    if mode == .FREE {
        assert(pool.active && old_memory == pool.block);
        pool.active = false;
        return null;
    }

    assert(false, "unexpected pool operation");

    return null;
}
