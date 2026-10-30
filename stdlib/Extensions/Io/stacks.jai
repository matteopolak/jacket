// Fiber stacks: one anonymous mapping each, with the lowest pages made inaccessible so that a fiber
// that overflows its stack faults instead of writing into its neighbour's memory.

Stack :: struct {
    // Lowest address of the mapping, which is the start of the guard region.
    base: *void;
    // Bytes mapped, guard included.
    size: s64;
    guard: s64;
}

#if OS == .WINDOWS {
    kernel32 :: #system_library "kernel32"; @native(external)
    VirtualAlloc :: (address: *void, size: u64, allocation_type: u32, protect: u32) -> *void #foreign kernel32;
    VirtualProtect :: (address: *void, size: u64, new_protect: u32, old_protect: *u32) -> s32 #foreign kernel32;
    VirtualFree :: (address: *void, size: u64, free_type: u32) -> s32 #foreign kernel32;
    DiscardVirtualMemory :: (address: *void, size: u64) -> u32 #foreign kernel32;

    MEM_COMMIT :: 0x1000;
    MEM_RESERVE :: 0x2000;
    MEM_RELEASE :: 0x8000;
    PAGE_NOACCESS :: 0x01;
    PAGE_READWRITE :: 0x04;

    // Windows pages are 4 KiB on x64 and arm64 alike.
    page_bytes :: () -> s64 {
        return 4096;
    }

    map_stack :: (size: s64, guard: s64) -> *void {
        base := VirtualAlloc(null, cast(u64) size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        if !base return null;

        previous: u32;
        if !VirtualProtect(base, cast(u64) guard, PAGE_NOACCESS, *previous) {
            VirtualFree(base, 0, MEM_RELEASE);
            return null;
        }

        return base;
    }

    unmap_stack :: (base: *void, size: s64) {
        VirtualFree(base, 0, MEM_RELEASE);
    }

    discard_pages :: (address: *void, size: s64) {
        DiscardVirtualMemory(address, cast(u64) size);
    }
} else {
    libc :: #system_library "libc"; @native(external)
    getpagesize :: () -> s32 #foreign libc;
    mmap :: (address: *void, length: u64, protection: s32, flags: s32, descriptor: s32, offset: s64) -> *void #foreign libc;
    mprotect :: (address: *void, length: u64, protection: s32) -> s32 #foreign libc;
    munmap :: (address: *void, length: u64) -> s32 #foreign libc;
    madvise :: (address: *void, length: u64, advice: s32) -> s32 #foreign libc;

    page_bytes :: () -> s64 {
        return cast(s64) getpagesize();
    }

    map_stack :: (size: s64, guard: s64) -> *void {
        // MAP_PRIVATE | MAP_ANONYMOUS, not reserving swap (MAP_NORESERVE); Linux also gets MAP_STACK.
        #if OS == .MACOS flags: s32 = 0x1002 | 0x40;
        else flags: s32 = 0x22 | 0x4000 | 0x20000;
        base := mmap(null, cast(u64) size, 3, flags, -1, 0);
        if base == cast(*void) -1 return null;

        if mprotect(base, cast(u64) guard, 0) != 0 {
            munmap(base, cast(u64) size);
            return null;
        }

        return base;
    }

    unmap_stack :: (base: *void, size: s64) {
        munmap(base, cast(u64) size);
    }

    // The pages stay mapped and read back as zeros; the system takes their memory back at once.
    discard_pages :: (address: *void, size: s64) {
        // macOS keeps the pages of a madvise range resident until it is short of memory, so a fresh
        // zero-fill mapping goes over the old one instead.
        #if OS == .MACOS mmap(address, cast(u64) size, 3, 0x1002 | 0x40 | 0x10, -1, 0);
        else madvise(address, cast(u64) size, 4); // MADV_DONTNEED
    }
}

// A stack of at least `bytes` bytes (rounded up to whole pages), or a zero `base`.
stack_allocate :: (bytes: s64) -> Stack {
    page := page_bytes();
    guard := max(page, 16384);
    guard = (guard + page - 1) / page * page;

    size := (max(bytes, guard + page * 4) + page - 1) / page * page;
    base := map_stack(size, guard);

    stack: Stack;

    if base stack = .{ base, size, guard };

    return stack;
}

stack_release :: (stack: Stack) {
    if stack.base unmap_stack(stack.base, stack.size);
} @FreesArgument

// How far below the caller's frame a discard starts. The calls that discard run in that space, so it
// must stay untouched. 2 KiB is enough on macOS and Linux; `DiscardVirtualMemory` goes deeper than
// that into the stack (a margin of 2 KiB crashed on Windows, 6 KiB did not).
#if OS == .WINDOWS {
    DISCARD_MARGIN :: 16384;
} else {
    DISCARD_MARGIN :: 2048;
}

// Gives the memory of the stack's whole pages that lie `DISCARD_MARGIN` bytes or more below `frame`
// back to the system. The pages stay mapped (and the guard untouched); their next use finds zeros on
// macOS and Linux and unspecified contents on Windows. `frame` is an address in the frame of the
// running procedure: everything below it must be dead, and the margin is left for the calls that
// do this.
stack_discard_below :: (stack: Stack, frame: *void) {
    page := page_bytes();
    low := cast(s64) stack.base + stack.guard;
    high := (cast(s64) frame - DISCARD_MARGIN) / page * page;

    if high <= low || high > cast(s64) stack.base + stack.size return;

    discard_pages(cast(*void) low, high - low);
}
