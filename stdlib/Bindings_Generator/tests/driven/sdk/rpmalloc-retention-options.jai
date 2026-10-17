Limited :: #import "rpmalloc"(GLOBAL_CACHE_MULTIPLIER = 1);
Unlimited :: #import "rpmalloc"(GLOBAL_CACHE_MULTIPLIER = 1, ENABLE_UNLIMITED_CACHE = true);
Retained :: #import "rpmalloc"(GLOBAL_CACHE_MULTIPLIER = 1, ENABLE_UNLIMITED_CACHE = true, DISABLE_UNMAP = true);
HugeOption :: #import "rpmalloc"(LINUX_HUGE_PAGES = true);

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

fill :: (proc: Allocator_Proc) {
    slots: [320] *void;
    for *slots {
        it.* = proc(.ALLOCATE, 131072, 0, null, null);
        check(it.* != null);
        (cast(*u8) it.*)[0] = 71;
    }

    for slots {
        check((cast(*u8) it)[0] == 71);
        proc(.FREE, 0, 0, it, null);
    }

    proc(.THREAD_STOP, 0, 0, null, null);
}

main :: () {
    check(Limited.initialize() == 0);
    fill(Limited.rpmalloc_allocator_proc);
    limited: Limited.rpmalloc_global_statistics_t;

    Limited.global_statistics(*limited);
    Limited.finalize();
    check(Unlimited.initialize() == 0);
    fill(Unlimited.rpmalloc_allocator_proc);
    unlimited: Unlimited.rpmalloc_global_statistics_t;

    Unlimited.global_statistics(*unlimited);
    check(unlimited.cached > limited.cached);
    Unlimited.finalize();
    check(Retained.initialize() == 0);
    fill(Retained.rpmalloc_allocator_proc);
    Retained.finalize();
    check(HugeOption.initialize() == 0);
    memory := HugeOption.rpmalloc(71);

    check(memory != null);
    HugeOption.rpfree(memory);
    HugeOption.finalize();
}
