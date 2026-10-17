// Default heap blocks that start on a cache line from 64 bytes up; smaller ones keep malloc's
// alignment. A type declared `#align 64` has a size that is a multiple of 64, so plain `New` and
// `alloc` honor it although the allocator protocol carries no alignment.
//
// Only malloc, realloc, memmove and free are used: compile-time code of a cross build calls the
// host's C library, so posix_memalign or _aligned_malloc would each be missing on some host and
// target pair. A block is a region malloc returned, `prefix` bytes of bookkeeping first, then the
// contents. The word just before the contents holds the region's start, which realloc and free
// want back. Callers put their own bookkeeping in the `prefix` bytes and keep that word last.
#scope_module
CACHE_LINE :: 64;

// The most the contents can be pushed forward to reach a cache line, whatever malloc's alignment.
SLACK :: CACHE_LINE - 8;

obtain_contents :: (size: s64, prefix: s64) -> *void {
    if size < 0 || size > 0x7fffffffffffffff - prefix - SLACK return null;

    wide := size >= CACHE_LINE;
    region := c_malloc(cast(u64) (size + prefix + ifx wide then SLACK else 0));

    if !region return null;

    contents := contents_in(region, prefix, wide);

    set_region(contents, region);

    return contents;
}

release_contents :: (contents: *void) {
    if contents c_free(region_of(contents));
}

// Resize the block at `contents` to `size` bytes. A block that is wide, or that grows to a cache
// line, ends up on a cache-line boundary; one that shrinks keeps its place. The contents move
// with the region, but the bookkeeping in the `prefix` bytes before them does not: the caller
// restores it around the returned address. On failure the block is untouched.
regrow_contents :: (contents: *void, size: s64, prefix: s64) -> *void {
    if size < 0 || size > 0x7fffffffffffffff - prefix - SLACK return null;

    region := region_of(contents);
    was_at := cast(s64) contents - cast(s64) region;
    wide := size >= CACHE_LINE;
    padded := wide || was_at != prefix;

    fresh := c_realloc(region, cast(u64) (size + prefix + ifx padded then SLACK else 0));

    if !fresh return null;

    at := was_at;
    if wide at = cast(s64) contents_in(fresh, prefix, true) - cast(s64) fresh;

    // The slack covers the largest shift, so `size` bytes fit at both places.
    if at != was_at c_memmove(cast(*u8) fresh + at, cast(*u8) fresh + was_at, cast(u64) size);

    result := cast(*u8) fresh + at;

    set_region(result, fresh);

    return result;
}

#scope_file
#import "internal/libc";

contents_in :: (region: *void, prefix: s64, wide: bool) -> *void {
    start := cast(u64) region + cast(u64) prefix;
    if wide start = (start + CACHE_LINE - 1) & ~cast(u64) (CACHE_LINE - 1);

    return cast(*void) start;
}

region_of :: (contents: *void) -> *void {
    return (cast(**void) (cast(*u8) contents - 8)).*;
}

set_region :: (contents: *void, region: *void) {
    (cast(**void) (cast(*u8) contents - 8)).* = region;
}
