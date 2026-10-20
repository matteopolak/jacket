// Shared by ordinary Runtime writes and the Android deferred-byte queue.
acquire :: () #no_context {
    acquired := compare_and_swap(*state, cast(s32) 0, cast(s32) 1);
    while !acquired acquired = compare_and_swap(*state, cast(s32) 0, cast(s32) 1);
}

release :: () -> bool #no_context {
    released := compare_and_swap(*state, cast(s32) 1, cast(s32) 0);
    return released;
}

#scope_file
state: s32;
