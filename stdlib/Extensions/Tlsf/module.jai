// A two-level segregated fit allocator: allocation and free take constant time, whatever the pool holds.
//
// A `Tlsf` manages one or more fixed pools of memory, either handed to `init_in_memory` or taken from a backing
// allocator by `init`. Free blocks are filed by size in a two-level table, so a request finds a suitable block with two
// bit scans, and freed neighbours merge at once. It never calls the system after setup, which makes its timing steady;
// the price is a 16-byte header per allocation and a fixed capacity: when no block fits, allocation returns null.
// Blocks are 16-byte aligned.
//
// ```jai
// #import "Extensions/Tlsf";
//
// tlsf: Tlsf;
// assert(init(*tlsf, 1024 * 1024));
// defer fini(*tlsf);
//
// context.allocator = as_allocator(*tlsf);
// numbers := NewArray(100, int);
// assert(numbers.count == 100);
// array_free(numbers);
// ```

CAPS_VERSION_STRING :: "Extensions/Tlsf";

// The most pools one `Tlsf` can manage.
MAX_POOLS :: 8;

// The alignment of every block, and the granularity of block sizes.
BLOCK_ALIGNMENT :: 16;

// One pool of memory a `Tlsf` manages.
Tlsf_Pool :: struct {
    start: *u8;
    end: *u8;
}

// The allocator's state. Keep it at a fixed address while an `Allocator` refers to it. A zeroed `Tlsf` is not ready:
// call `init` or `init_in_memory`.
//
// `thread_safe` makes `tlsf_proc` take a spin lock around each call, so one `Tlsf` can serve several threads through
// `context.allocator`. The procedures `allocate`, `release` and the others do not lock; call them from one thread or
// hold your own lock.
Tlsf :: struct {
    thread_safe: bool;

    first_level: u32;
    second_level: [FIRST_LEVELS] u32;
    free_lists: [FIRST_LEVELS * SECOND_LEVELS] *Block;

    pools: [MAX_POOLS] Tlsf_Pool;
    pool_count: s32;

    owned_memory: *void;
    backing: Allocator;
    lock: s32;
}

// Block counts and sizes of a `Tlsf`, in bytes of block (headers included).
Tlsf_Statistics :: struct {
    pool_bytes: s64;
    used_bytes: s64;
    free_bytes: s64;
    used_blocks: s64;
    free_blocks: s64;
    largest_free_payload: s64; // the biggest request that would succeed now
}

// Sets up a pool of `bytes` taken from `backing`, which `fini` gives back. Returns false when the backing allocator
// fails or `bytes` is too small or too large (at most 256 GiB).
//
// ```jai
// tlsf: Tlsf;
// assert(init(*tlsf, 64 * 1024, thread_safe = true));
// defer fini(*tlsf);
// assert(allocate(*tlsf, 100) != null);
// ```
init :: (tlsf: *Tlsf, bytes: s64, thread_safe := false, backing := context.allocator) -> bool {
    assert(tlsf.pool_count == 0, "tlsf is already initialized");
    if bytes <= 0 || bytes > MAX_BLOCK return false;

    memory := backing.proc(.ALLOCATE, bytes + BLOCK_ALIGNMENT, 0, null, backing.data);
    if !memory return false;

    if !init_in_memory(tlsf, memory, bytes + BLOCK_ALIGNMENT, thread_safe) {
        backing.proc(.FREE, 0, 0, memory, backing.data);
        return false;
    }

    tlsf.owned_memory = memory;
    tlsf.backing = backing;

    return true;
}

// Sets up a pool inside `memory`, which the caller keeps owning and must not touch while the `Tlsf` uses it. Returns
// false when the range is too small (under 64 bytes) or too large.
//
// ```jai
// storage: [4096] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// block := allocate(*tlsf, 100);
// assert(block != null && owns(*tlsf, block));
// ```
init_in_memory :: (tlsf: *Tlsf, memory: *void, bytes: s64, thread_safe := false) -> bool {
    assert(tlsf.pool_count == 0, "tlsf is already initialized");
    tlsf.thread_safe = thread_safe;
    return add_pool(tlsf, memory, bytes);
}

// Gives the `Tlsf` another range of memory to allocate from. Returns false when the range is too small or too large, or
// `MAX_POOLS` pools exist. Blocks never span pools.
//
// ```jai
// first: [1024] u8;
// second: [1024] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, first.data, first.count));
// assert(add_pool(*tlsf, second.data, second.count));
// ```
add_pool :: (tlsf: *Tlsf, memory: *void, bytes: s64) -> bool {
    if tlsf.pool_count >= MAX_POOLS || bytes <= 0 return false;

    start := align_up(cast(u64) memory, BLOCK_ALIGNMENT);
    end := align_down(cast(u64) memory + cast(u64) bytes, BLOCK_ALIGNMENT);

    if end <= start || end - start < 2 * HEADER_SIZE + MIN_BLOCK || end - start - HEADER_SIZE > cast(u64) MAX_BLOCK return false;

    size := end - start - HEADER_SIZE;
    block := cast(*Block) start;
    block.previous = null;
    block.size_and_state = size | FREE;

    sentinel := cast(*Block) (start + size); // a used block of size zero ends the pool
    sentinel.previous = block;
    sentinel.size_and_state = 0;

    tlsf.pools[tlsf.pool_count] = .{ cast(*u8) start, cast(*u8) end };
    tlsf.pool_count += 1;
    insert_free(tlsf, block);

    return true;
}

// Forgets every pool and returns the memory `init` took from its backing allocator. All blocks become invalid.
//
// ```jai
// tlsf: Tlsf;
// assert(init(*tlsf, 4096));
// fini(*tlsf);
// ```
fini :: (tlsf: *Tlsf) {
    if tlsf.owned_memory tlsf.backing.proc(.FREE, 0, 0, tlsf.owned_memory, tlsf.backing.data);
    tlsf.* = .{};
}

// Returns a block of at least `size` bytes, aligned to 16, or null when no free block is big enough. The memory is not
// cleared. To stay constant-time the search skips the one size class `size` falls in, so near exhaustion a request can fail
// while a free block less than 1/16 larger than it exists (never for blocks under 256 bytes).
//
// ```jai
// storage: [1024] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// block := allocate(*tlsf, 100);
// assert(block != null && allocation_size(block) >= 100);
// assert(allocate(*tlsf, 5000) == null);
// ```
allocate :: (tlsf: *Tlsf, size: s64) -> *void {
    if size < 0 || size > MAX_BLOCK return null;

    block := take_block(tlsf, block_size_for(size));
    if !block return null;
    return payload_of(block);
}

// Like `allocate`, with the block aligned to `alignment`, a power of two.
//
// ```jai
// storage: [8192] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// block := allocate_aligned(*tlsf, 100, 256);
// assert(block != null && (cast(u64) block & 255) == 0);
// release(*tlsf, block);
// ```
allocate_aligned :: (tlsf: *Tlsf, size: s64, alignment: s64) -> *void {
    assert(alignment > 0 && (alignment & (alignment - 1)) == 0, "alignment must be a power of two");
    if alignment <= BLOCK_ALIGNMENT return allocate(tlsf, size);
    if size < 0 || size > MAX_BLOCK || alignment > MAX_BLOCK return null;

    // Ask for room to slide the payload up to the alignment, then give the leading gap back as its own free block.
    block := take_block(tlsf, block_size_for(size + alignment + MIN_BLOCK));
    if !block return null;

    payload := cast(u64) payload_of(block);
    aligned := align_up(payload, alignment);
    gap := aligned - payload;

    if gap != 0 && gap < MIN_BLOCK {
        aligned = align_up(payload + MIN_BLOCK, alignment);
        gap = aligned - payload;
    }

    if gap != 0 {
        used := split_block(block, gap);
        set_free(block);
        insert_free(tlsf, block);
        block = used;
    }

    trim_block(tlsf, block, block_size_for(size));

    return payload_of(block);
}

// Makes a block free again, merging it with free neighbours. Null is ignored.
//
// ```jai
// storage: [1024] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// block := allocate(*tlsf, 100);
// release(*tlsf, block);
// assert(statistics(*tlsf).used_blocks == 0);
// ```
release :: (tlsf: *Tlsf, memory: *void) {
    if !memory return;

    block := header_of(memory);

    assert(!is_free(block), "tlsf block released twice");
    set_free(block);

    previous := block.previous;
    if previous && is_free(previous) {
        remove_free(tlsf, previous);
        merge_into_previous(block);
        block = previous;
    }

    next := next_block(block);
    if is_free(next) {
        remove_free(tlsf, next);
        absorb_next(block);
    }

    insert_free(tlsf, block);
}

// Changes a block's size to `new_size`, keeping its contents up to the smaller size. A block grows or shrinks in place
// when the space after it allows; otherwise the contents move. Returns null and leaves the old block valid when no
// room is found. A null `memory` allocates.
//
// ```jai
// storage: [4096] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// block := allocate(*tlsf, 100);
// bigger := resize(*tlsf, block, 1000);
// assert(bigger == block); // the free space after it was used
// ```
resize :: (tlsf: *Tlsf, memory: *void, new_size: s64) -> *void {
    if !memory return allocate(tlsf, new_size);
    if new_size < 0 || new_size > MAX_BLOCK return null;

    block := header_of(memory);

    assert(!is_free(block), "tlsf block resized after release");
    wanted := block_size_for(new_size);
    current := block_size(block);

    if wanted <= current {
        trim_block(tlsf, block, wanted);
        return memory;
    }

    next := next_block(block);
    if is_free(next) && current + block_size(next) >= wanted {
        remove_free(tlsf, next);
        absorb_next(block);
        trim_block(tlsf, block, wanted);
        return memory;
    }

    moved := allocate(tlsf, new_size);
    if !moved return null;

    memcpy(moved, memory, min(cast(s64) current - HEADER_SIZE, new_size));
    release(tlsf, memory);

    return moved;
}

// The bytes a block from `allocate` can hold: at least what was asked for.
//
// ```jai
// storage: [1024] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// assert(allocation_size(allocate(*tlsf, 10)) == 16);
// ```
allocation_size :: (memory: *void) -> s64 {
    return cast(s64) (block_size(header_of(memory)) - HEADER_SIZE);
}

// Whether `memory` lies inside one of the pools.
//
// ```jai
// storage: [1024] u8;
// local: int;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// assert(owns(*tlsf, allocate(*tlsf, 8)));
// assert(!owns(*tlsf, *local));
// ```
owns :: (tlsf: *Tlsf, memory: *void) -> bool {
    address := cast(*u8) memory;

    for 0..tlsf.pool_count - 1 {
        pool := tlsf.pools[it];
        if address >= pool.start + HEADER_SIZE && address < pool.end - HEADER_SIZE return true;
    }

    return false;
}

// Counts the blocks and bytes of the pools by walking them.
//
// ```jai
// storage: [1024] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// allocate(*tlsf, 100);
// stats := statistics(*tlsf);
// assert(stats.used_blocks == 1 && stats.free_blocks == 1);
// ```
statistics :: (tlsf: *Tlsf) -> Tlsf_Statistics {
    stats: Tlsf_Statistics;

    for 0..tlsf.pool_count - 1 {
        pool := tlsf.pools[it];

        stats.pool_bytes += pool.end - pool.start;

        block := cast(*Block) pool.start;
        while block_size(block) != 0 {
            size := cast(s64) block_size(block);

            if is_free(block) {
                stats.free_blocks += 1;
                stats.free_bytes += size;
                stats.largest_free_payload = max(stats.largest_free_payload, size - HEADER_SIZE);
            } else {
                stats.used_blocks += 1;
                stats.used_bytes += size;
            }

            block = next_block(block);
        }
    }

    return stats;
}

// Checks the pools and the free tables for corruption: block sizes, neighbour links, merged neighbours and that every
// free block is filed under its size. Returns false on the first problem. It walks everything, so use it in tests and
// when chasing a bug.
//
// ```jai
// storage: [1024] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// allocate(*tlsf, 100);
// assert(check(*tlsf));
// ```
check :: (tlsf: *Tlsf) -> bool {
    walked_free: s64;

    for 0..tlsf.pool_count - 1 {
        pool := tlsf.pools[it];
        block := cast(*Block) pool.start;
        previous: *Block;

        while block_size(block) != 0 {
            if (block_size(block) & (BLOCK_ALIGNMENT - 1)) != 0 || block_size(block) < MIN_BLOCK return false;
            if block.previous != previous return false;
            if previous && is_free(previous) && is_free(block) return false;
            if is_free(block) walked_free += 1;

            previous = block;
            block = next_block(block);
            if cast(*u8) block >= pool.end return false;
        }

        if cast(*u8) block != pool.end - HEADER_SIZE || block.previous != previous || is_free(block) return false;
    }

    listed_free: s64;

    for list: 0..FIRST_LEVELS * SECOND_LEVELS - 1 {
        first := list / SECOND_LEVELS;
        second := list % SECOND_LEVELS;
        head := tlsf.free_lists[list];

        has_bit := (tlsf.second_level[first] & (cast(u32) 1 << cast(u32) second)) != 0;

        if has_bit != (head != null) return false;
        if has_bit && (tlsf.first_level & (cast(u32) 1 << cast(u32) first)) == 0 return false;

        previous: *Block;
        block := head;

        while block {
            if !is_free(block) || block.previous_free != previous return false;

            fl, sl := mapping_insert(block_size(block));
            if fl != first || sl != second return false;

            listed_free += 1;
            previous = block;
            block = block.next_free;
        }
    }

    return walked_free == listed_free;
}

// This `Tlsf` as an allocator for `context.allocator`.
//
// ```jai
// storage: [4096] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// context.allocator = as_allocator(*tlsf);
// text := copy_string("hello");
// assert(text == "hello");
// ```
as_allocator :: (tlsf: *Tlsf) -> Allocator {
    return .{ tlsf_proc, tlsf };
}

// The `Allocator_Proc` of a `Tlsf`, used as `Allocator.{ tlsf_proc, *tlsf }`. A failed allocation returns null. With
// `thread_safe` set, each call holds the spin lock.
//
// ```jai
// storage: [1024] u8;
// tlsf: Tlsf;
// assert(init_in_memory(*tlsf, storage.data, storage.count));
// block := tlsf_proc(.ALLOCATE, 64, 0, null, *tlsf);
// tlsf_proc(.FREE, 0, 0, block, *tlsf);
// ```
tlsf_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    tlsf := cast(*Tlsf) allocator_data;

    assert(tlsf != null, "the tlsf allocator needs its Tlsf as data");

    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = CAPS_VERSION_STRING;
        }

        caps := Allocator_Caps.FREE | .ACTUALLY_RESIZE | .IS_THIS_YOURS | .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR;
        if tlsf.thread_safe caps |= .MULTIPLE_THREADS;

        return cast(*void) cast(u64) caps;
    }

    if mode != .ALLOCATE && mode != .RESIZE && mode != .FREE && mode != .IS_THIS_YOURS return null;

    if tlsf.thread_safe lock(tlsf);

    defer if tlsf.thread_safe unlock(tlsf);

    if mode == .ALLOCATE return allocate(tlsf, requested_size);
    if mode == .RESIZE return resize(tlsf, old_memory, requested_size);

    if mode == .FREE {
        release(tlsf, old_memory);
        return null;
    }

    if old_memory && owns(tlsf, old_memory) return cast(*void) 1;

    return null;
}

#scope_file

#import "Basic";
#import "Atomics";
#import "Bit_Operations";

// A block is a 16-byte header followed by its payload. `previous` is the block before it in memory, and `size_and_state`
// is the block's whole size, header included, with the low bit set while it is free. A free block's payload holds the
// links of its free list. Every pool ends in a used block of size zero, so a block always has a next neighbour.
Block :: struct {
    previous: *Block;
    size_and_state: u64;
    next_free: *Block;
    previous_free: *Block;
}

HEADER_SIZE :: 16;
MIN_BLOCK :: 32;
MAX_BLOCK :: 1 << 38;
FREE :: 1;
ALL_BITS: u32 : 0xffffffff;

// Sizes below SMALL_LIMIT are filed by 16-byte steps in the first row; above, each power of two is split into
// SECOND_LEVELS rows.
SECOND_LEVEL_LOG2 :: 4;
SECOND_LEVELS :: 1 << SECOND_LEVEL_LOG2;
SMALL_LIMIT :: 256;
FIRST_LEVEL_SHIFT :: 8;
FIRST_LEVELS :: 32;

align_up :: (value: u64, alignment: s64) -> u64 {
    mask := cast(u64) alignment - 1;
    return (value + mask) & ~mask;
}

align_down :: (value: u64, alignment: s64) -> u64 {
    return value & ~(cast(u64) alignment - 1);
}

block_size :: (block: *Block) -> u64 {
    return block.size_and_state & ~cast(u64) (BLOCK_ALIGNMENT - 1);
}

is_free :: (block: *Block) -> bool {
    return (block.size_and_state & FREE) != 0;
}

set_free :: (block: *Block) {
    block.size_and_state |= FREE;
}

set_size :: (block: *Block, size: u64) {
    block.size_and_state = size | (block.size_and_state & FREE);
}

next_block :: (block: *Block) -> *Block {
    return cast(*Block) (cast(*u8) block + block_size(block));
}

payload_of :: (block: *Block) -> *void {
    return cast(*u8) block + HEADER_SIZE;
}

header_of :: (memory: *void) -> *Block {
    return cast(*Block) (cast(*u8) memory - HEADER_SIZE);
}

// The block size that holds `payload` bytes.
block_size_for :: (payload: s64) -> u64 {
    return max(align_up(cast(u64) payload, BLOCK_ALIGNMENT) + HEADER_SIZE, MIN_BLOCK);
}

// The first-level and second-level row of a block of this size.
mapping_insert :: (size: u64) -> s32, s32 {
    if size < SMALL_LIMIT return 0, cast(s32) (size >> 4);

    msb := bit_scan_reverse(size) - 1;
    second := cast(s32) ((size >> cast(u64) (msb - SECOND_LEVEL_LOG2)) ^ SECOND_LEVELS);
    return msb - (FIRST_LEVEL_SHIFT - 1), second;
}

// The row to search for a block of at least `size`: round up so every block listed there is big enough.
mapping_search :: (size: u64) -> s32, s32 {
    rounded := size;
    if rounded >= SMALL_LIMIT {
        msb := bit_scan_reverse(rounded) - 1;
        rounded += (cast(u64) 1 << cast(u64) (msb - SECOND_LEVEL_LOG2)) - 1;
    }

    return mapping_insert(rounded);
}

insert_free :: (tlsf: *Tlsf, block: *Block) {
    first, second := mapping_insert(block_size(block));
    head := tlsf.free_lists[first * SECOND_LEVELS + second];

    block.next_free = head;
    block.previous_free = null;
    if head head.previous_free = block;

    tlsf.free_lists[first * SECOND_LEVELS + second] = block;
    tlsf.first_level |= cast(u32) 1 << cast(u32) first;
    tlsf.second_level[first] |= cast(u32) 1 << cast(u32) second;
}

remove_free :: (tlsf: *Tlsf, block: *Block) {
    first, second := mapping_insert(block_size(block));
    next := block.next_free;
    previous := block.previous_free;

    if next next.previous_free = previous;

    if previous {
        previous.next_free = next;
        return;
    }

    tlsf.free_lists[first * SECOND_LEVELS + second] = next;
    if next return;

    tlsf.second_level[first] &= ~(cast(u32) 1 << cast(u32) second);
    if tlsf.second_level[first] == 0 tlsf.first_level &= ~(cast(u32) 1 << cast(u32) first);
}

find_suitable :: (tlsf: *Tlsf, size: u64) -> *Block {
    first, second := mapping_search(size);
    if first >= FIRST_LEVELS return null;

    seconds := tlsf.second_level[first] & (ALL_BITS << cast(u32) second);

    if seconds == 0 {
        if first + 1 >= FIRST_LEVELS return null;

        firsts := tlsf.first_level & (ALL_BITS << cast(u32) (first + 1));
        if firsts == 0 return null;

        first = bit_scan_forward(firsts) - 1;
        seconds = tlsf.second_level[first];
    }

    second = bit_scan_forward(seconds) - 1;

    return tlsf.free_lists[first * SECOND_LEVELS + second];
}

// Cuts `block` into one of `size` bytes and a block of the rest, which is returned and has no state bits set.
split_block :: (block: *Block, size: u64) -> *Block {
    rest := cast(*Block) (cast(*u8) block + size);
    rest.previous = block;
    rest.size_and_state = block_size(block) - size;

    set_size(block, size);
    next_block(rest).previous = rest;

    return rest;
}

// Merges `block` into the block before it; `block` must not be used afterwards.
merge_into_previous :: (block: *Block) {
    previous := block.previous;
    set_size(previous, block_size(previous) + block_size(block));
    next_block(previous).previous = previous;
}

// Merges the next block into `block`.
absorb_next :: (block: *Block) {
    next := next_block(block);
    set_size(block, block_size(block) + block_size(next));
    next_block(block).previous = block;
}

// Gives the end of a used block beyond `size` bytes back as free space.
trim_block :: (tlsf: *Tlsf, block: *Block, size: u64) {
    if block_size(block) - size < MIN_BLOCK return;

    rest := split_block(block, size);
    rest.size_and_state |= FREE;

    next := next_block(rest);
    if is_free(next) {
        remove_free(tlsf, next);
        absorb_next(rest);
    }

    insert_free(tlsf, rest);
}

// Removes a free block of at least `size` from the table and marks it used, keeping only what is needed.
take_block :: (tlsf: *Tlsf, size: u64) -> *Block {
    block := find_suitable(tlsf, size);
    if !block return null;

    remove_free(tlsf, block);
    block.size_and_state &= ~cast(u64) FREE;
    trim_block(tlsf, block, size);

    return block;
}

lock :: (tlsf: *Tlsf) {
    while atomic_swap(*tlsf.lock, cast(s32) 1) != 0 {}
}

unlock :: (tlsf: *Tlsf) {
    atomic_write(*tlsf.lock, cast(s32) 0);
}
