// Native SDK alignment is a caller precondition. Keep invalid requests out of
#module_parameters (STATISTICS := true, RAW := false, PUBLIC_HEAPS := true, GLOBAL_CACHE := true, THREAD_CACHE := true, ASSERTS := true, VALIDATE_ARGS := true, ADAPTIVE := false)();

// it, and keep the output slot unchanged when its allocation attempt fails.
// Requires initialized native SDK/thread state; no public Jai config is cast.
posix_memalign :: (output: **void, alignment: u64, size: u64) -> s32 {
    if !output || alignment < size_of(*void) || (alignment & (alignment - 1)) != 0 return EINVAL;
    if alignment >= SDK.rpmalloc_config().span_size return EINVAL;

    allocation: *void;
    result := SDK.rpposix_memalign(*allocation, alignment, size);

    if result == 0 output.* = allocation;

    return result;
}

#scope_file
SDK :: #import "internal/bindings/rpmalloc-sdk"(STATISTICS = STATISTICS, RAW = RAW, PUBLIC_HEAPS = PUBLIC_HEAPS, GLOBAL_CACHE = GLOBAL_CACHE, THREAD_CACHE = THREAD_CACHE, ASSERTS = ASSERTS, VALIDATE_ARGS = VALIDATE_ARGS, ADAPTIVE = ADAPTIVE);

// Reviewed public errno value and native C ABI value on the six desktop targets.
EINVAL :: 22;
