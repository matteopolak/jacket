#add_context domain_marker: s64;

Small :: #import "rpmalloc"(HEAP_ARRAY_SIZE = 17, DEFAULT_SPAN_MAP_COUNT = 3, GLOBAL_CACHE_MULTIPLIER = 2);
Same :: #import "rpmalloc"(HEAP_ARRAY_SIZE = 17, DEFAULT_SPAN_MAP_COUNT = 3, GLOBAL_CACHE_MULTIPLIER = 2);
Large :: #import "rpmalloc"(HEAP_ARRAY_SIZE = 53, DEFAULT_SPAN_MAP_COUNT = 96, GLOBAL_CACHE_MULTIPLIER = 3);
Default :: #import "rpmalloc";
ExplicitDefault :: #import "rpmalloc"(HEAP_ARRAY_SIZE = 47);
RawSmall :: #import "rpmalloc"(USE_RAW = true, HEAP_ARRAY_SIZE = 17, DEFAULT_SPAN_MAP_COUNT = 3);
RawLarge :: #import "rpmalloc"(USE_RAW = true, HEAP_ARRAY_SIZE = 53, DEFAULT_SPAN_MAP_COUNT = 96);

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

small_map: #type (size: s64, offset: *s64) -> *void;
large_map: #type (size: s64, offset: *s64) -> *void;
small_maps: s64;
large_maps: s64;

map_small :: (size: s64, offset: *s64) -> *void {
    check(context.domain_marker == 713);
    small_maps += 1;
    return small_map(size, offset);
}

map_large :: (size: s64, offset: *s64) -> *void {
    check(context.domain_marker == 713);
    large_maps += 1;
    return large_map(size, offset);
}

main :: () {
    context.domain_marker = 713;
    check(Small.config() == Same.config());
    check(cast(*void) Small.config() != cast(*void) Large.config());
    check(cast(*void) Default.config() != cast(*void) ExplicitDefault.config());
    check(Small.initialize() == 0 && Large.initialize() == 0);
    check(Same.is_initialized());
    check(Small.config().span_map_count == 3 && Large.config().span_map_count == 96);
    small_map = Small.config().memory_map;
    large_map = Large.config().memory_map;
    Small.config().memory_map = map_small;
    Large.config().memory_map = map_large;
    first := Small.rpmalloc(33_554_432);

    check(small_maps > 0 && large_maps == 0);
    second := Large.rpmalloc(33_554_432);

    check(large_maps > 0);

    check(first != null && second != null && first != second);
    (cast(*u8) first)[0] = 71;
    (cast(*u8) second)[0] = 74;
    Small.rpfree(first);
    Small.finalize();
    check(!Same.is_initialized() && Large.is_initialized());
    check((cast(*u8) second)[0] == 74);
    second = Large.rprealloc(second, 67_108_864);
    check(second != null && (cast(*u8) second)[0] == 74);
    Large.rpfree(second);
    Large.finalize();
    check(Default.initialize() == 0 && ExplicitDefault.initialize() == 0);
    Default.finalize();
    check(ExplicitDefault.is_initialized());
    ExplicitDefault.finalize();
    check(RawSmall.rpmalloc_initialize() == 0 && RawLarge.rpmalloc_initialize() == 0);
    raw_first := RawSmall.rpmalloc(4096);
    raw_second := RawLarge.rpmalloc(4096);

    check(raw_first != null && raw_second != null);
    (cast(*u8) raw_second)[0] = 47;
    RawSmall.rpfree(raw_first);
    RawSmall.rpmalloc_finalize();
    check(RawLarge.rpmalloc_is_initialized() && (cast(*u8) raw_second)[0] == 47);
    RawLarge.rpfree(raw_second);
    RawLarge.rpmalloc_finalize();

    // Repeat init/finalize exercises domain/TLS ownership rather than leaking
    // one native allocator instance or accidentally sharing its predecessor.
    for 0..15 {
        check(Small.initialize() == 0 && Large.initialize() == 0);
        check(Small.config().span_map_count == 3 && Large.config().span_map_count == 96);
        Small.finalize();
        Large.finalize();
    }

    check(context.domain_marker == 713);
}
