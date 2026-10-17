POOL_BUCKET_SIZE_DEFAULT :: 65536;
POOL_OVERSIZED_SIZE_DEFAULT :: 6554;
STAMP :: 0xcc;
HEADER_SIZE :: size_of(Memblock);

Pool :: struct {
    memblock_size: s64 = POOL_BUCKET_SIZE_DEFAULT;
    oversized_size: s64 = POOL_OVERSIZED_SIZE_DEFAULT;
    alignment := 8;
    current_memblock: *Memblock;
    current_pos: *u8;
    bytes_left: s64;
    block_allocator: Allocator;
    unused_memblocks: *Memblock;
    used_memblocks: *Memblock;
    first_used_memblock: *Memblock;
    out_of_band_allocations: *Memblock;
    overwrite_memory := false;
    free_memblocks_on_reset := false;
    logging_options: enum_flags u32 {
        LOG_OVERSIZED_ALLOCATIONS :: 1;
        LOG_OVERSIZED_ALLOCATIONS_AS_ERRORS :: 2;
        LOG_OVERSIZED_ALLOCATIONS_STACK_TRACE :: 4;
    }
}

set_allocators :: (pool: *Pool, block_allocator: Allocator = .{}) {
    if pool.block_allocator.proc {
        context.logger("Pool allocator is already selected\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return;
    }

    pool.block_allocator = block_allocator;
    if !pool.block_allocator.proc {
        #if USE_UNMAPPING_ALLOCATOR {
            pool.block_allocator = U.get_unmapping_allocator();
            pool.free_memblocks_on_reset = true;
        } else pool.block_allocator = context.allocator;
    }
}

get :: (using pool: *Pool, nbytes: s64) -> *void {
    assert(nbytes >= 0 && alignment > 0 && (alignment & (alignment - 1)) == 0);
    // Include both public linked headers and the private backing-size word.
    // An unrepresentable extent cannot be passed to a signed-size allocator.
    if nbytes > 0x7fffffffffffffff - alignment - HEADER_SIZE - 8 return null;
    if !block_allocator.proc set_allocators(pool);

    if nbytes >= oversized_size || nbytes + alignment + HEADER_SIZE > memblock_size {
        block := allocate_block(pool, nbytes + alignment + HEADER_SIZE);
        if !block return null;

        block.next = out_of_band_allocations;
        out_of_band_allocations = block;

        if logging_options != 0 {
            flags := Log_Flags.NONE;
            if (logging_options & .LOG_OVERSIZED_ALLOCATIONS_AS_ERRORS) != 0 flags = .ERROR;

            context.logger("Pool oversized allocation\n", context.logger_data, Log_Info.{ common_flags = flags });

            if (logging_options & .LOG_OVERSIZED_ALLOCATIONS_STACK_TRACE) != 0 {
                node := context.stack_trace;
                while node {
                    if node.info write_strings("  ", node.info.name, "\n", to_standard_error = true);
                    node = node.next;
                }
            }
        }

        return cast(*void) align_forward(cast(s64) block + HEADER_SIZE, alignment);
    }

    aligned := align_forward(cast(s64) current_pos, alignment);
    skipped := aligned - cast(s64) current_pos;

    if !current_memblock || skipped + nbytes > bytes_left {
        block := unused_memblocks;
        if block unused_memblocks = block.next;
        else block = allocate_block(pool, memblock_size);
        if !block return null;

        block.next = used_memblocks;
        used_memblocks = block;
        if !first_used_memblock first_used_memblock = block;

        current_memblock = block;
        current_pos = cast(*u8) block + HEADER_SIZE;
        bytes_left = memblock_size - HEADER_SIZE;
        aligned = align_forward(cast(s64) current_pos, alignment);
        skipped = aligned - cast(s64) current_pos;
    }

    result := cast(*u8) aligned;

    bytes_left -= skipped + nbytes;
    current_pos = result + nbytes;

    return result;
}

reset :: (using pool: *Pool) {
    block := out_of_band_allocations;
    while block {
        next := block.next;
        free_block(pool, block);
        block = next;
    }

    out_of_band_allocations = null;
    block = used_memblocks;
    while block {
        next := block.next;

        if overwrite_memory memset(cast(*u8) block + HEADER_SIZE, STAMP, memblock_size - HEADER_SIZE);

        if free_memblocks_on_reset free_block(pool, block);
        else {
            block.next = unused_memblocks;
            unused_memblocks = block;
        }

        block = next;
    }

    used_memblocks = null;
    first_used_memblock = null;
    current_memblock = null;
    current_pos = null;
    bytes_left = 0;
}

release :: (using pool: *Pool) {
    reset(pool);
    block := unused_memblocks;
    while block {
        next := block.next;
        free_block(pool, block);
        block = next;
    }

    unused_memblocks = null;
} @Releases

calculate_used_memory :: (using pool: Pool) -> inband_size: s64, outband_count: s64 {
    block := used_memblocks;
    bytes: s64;

    while block {
        bytes += memblock_size;
        block = block.next;
    }

    block = out_of_band_allocations;
    count: s64;

    while block {
        count += 1;
        block = block.next;
    }

    return bytes - bytes_left, count;
}

pool_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = "Pool";
        }

        return cast(*void) cast(u64) (Allocator_Caps.IS_THIS_YOURS | .HINT_I_AM_A_FAST_BUMP_ALLOCATOR);
    }

    pool := cast(*Pool) allocator_data;

    assert(pool != null);
    if mode == .ALLOCATE return get(pool, requested_size);

    if mode == .RESIZE {
        result := get(pool, requested_size);
        if result && old_memory memcpy(result, old_memory, min(old_size, requested_size));
        return result;
    }

    if mode == .FREE return null;

    if mode == .STARTUP {
        if !pool.block_allocator.proc set_allocators(pool);
        return null;
    }

    if mode == .SHUTDOWN {
        release(pool);
        return null;
    }

    if mode == .IS_THIS_YOURS {
        block := pool.used_memblocks;
        while block {
            address := cast(u64) old_memory;
            if address >= cast(u64) block + HEADER_SIZE && address < cast(u64) block + cast(u64) pool.memblock_size return cast(*void) 1;
            block = block.next;
        }

        return null;
    }

    assert(false, "unsupported Pool allocator mode");

    return null;
}

example_usage :: () {
    pool: Pool;

    set_allocators(*pool);
    defer release(*pool);
    data := get(*pool, 32);

    assert(data != null);
    memset(data, 0, 32);
    reset(*pool);
}

#scope_file
#import "Basic";
#if USE_UNMAPPING_ALLOCATOR U :: #import "Unmapping_Allocator";

Memblock :: struct {
    next: *Memblock;
    data: void;
}

allocate_block :: (pool: *Pool, bytes: s64) -> *Memblock {
    if bytes < 0 || bytes > 0x7fffffffffffffff - 8 return null;

    raw := pool.block_allocator.proc(.ALLOCATE, bytes + 8, 0, null, pool.block_allocator.data);
    if !raw return null;

    (cast(*s64) raw).* = bytes + 8;

    return cast(*Memblock) (cast(*u8) raw + 8);
}

free_block :: (pool: *Pool, block: *Memblock) {
    raw := cast(*u8) block - 8;
    size := (cast(*s64) raw).*;
    pool.block_allocator.proc(.FREE, 0, size, raw, pool.block_allocator.data);
} @Releases
