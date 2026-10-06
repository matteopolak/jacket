// Allocators used from several threads at once: the default allocator's shared ledger, an
// Overwriting_Allocator (which locks its own table), and each thread's own temporary storage.
// Each thread keeps a set of live blocks filled with its own pattern and checks them before
// freeing, so a block handed to two threads, or lost by the ledger, is caught.
#import "Basic";
#import "Thread";
#import "Overwriting_Allocator";

THREADS :: 4;
ROUNDS :: 1500;
LIVE :: 32;

shared_debug_allocator: Allocator;

Worker_Result :: struct {
    checked: s64;
    temporary_ok: bool;
    corrupted: bool;
}

churn :: (thread: *Thread) -> s64 {
    result := cast(*Worker_Result) thread.data;
    seed := cast(u64) thread.index * 0x9e37_79b9 + 1;
    pattern := cast(u8) (thread.index * 37);
    blocks: [LIVE] *u8;
    sizes: [LIVE] s64;
    allocators: [LIVE] Allocator;
    note: string;
    for round: 0..ROUNDS - 1 {
        seed = seed * 6364136223846793005 + 1442695040888963407;
        slot := cast(s64) ((seed >> 33) % LIVE);
        if blocks[slot] {
            for i: 0..sizes[slot] - 1 {
                if blocks[slot][i] != pattern {
                    print("thread %: block in slot % changed at byte %\n", thread.index, slot, i, to_standard_error = true);
                    result.corrupted = true;
                    return 1;
                }
            }
            result.checked += 1;
            if round % 3 == 0 {
                // Grow it in place or by moving, then check the kept bytes again.
                new_size := sizes[slot] * 2 + 1;
                grown := cast(*u8) realloc(blocks[slot], new_size, sizes[slot], allocators[slot]);
                for i: 0..sizes[slot] - 1 if grown[i] != pattern {
                    result.corrupted = true;
                    return 2;
                }
                memset(grown, pattern, new_size);
                blocks[slot] = grown;
                sizes[slot] = new_size;
            } else {
                free(blocks[slot], allocators[slot]);
                blocks[slot] = null;
            }
        } else {
            size := cast(s64) ((seed >> 40) % 300) + 1;
            allocator := ifx round % 2 then shared_debug_allocator else context.allocator;
            block := cast(*u8) alloc(size, allocator);
            memset(block, pattern, size);
            blocks[slot] = block;
            sizes[slot] = size;
            allocators[slot] = allocator;
        }
        // Temporary storage is per thread: text printed at one checkpoint is unchanged at the next,
        // although the other threads have been printing into their own storage in between.
        if round % 50 == 0 {
            if note.count && note != expected_note(thread.index, round - 50) result.temporary_ok = false;
            reset_temporary_storage();
            note = expected_note(thread.index, round);
        }
    }
    for blocks if it free(it, allocators[it_index]);
    return 0;
}

expected_note :: (index: s64, round: s64) -> string {
    return tprint("thread % round %", index, round);
}

main :: () {
    shared_debug_allocator = get_overwriting_allocator();
    threads: [THREADS] Thread;
    results: [THREADS] Worker_Result;
    for *threads {
        it.data = *results[it_index];
        results[it_index].temporary_ok = true;
        assert(thread_init(it, churn));
    }
    for *threads assert(thread_start(it));
    for *threads thread_deinit(it);
    for results {
        assert(!it.corrupted, "thread % found a block changed by someone else", it_index);
        assert(it.checked > 0, "thread % checked nothing", it_index);
        assert(it.temporary_ok, "thread % saw its temporary storage change", it_index);
    }
    deinit_overwriting_allocator(shared_debug_allocator);
    print("ok\n");
}
