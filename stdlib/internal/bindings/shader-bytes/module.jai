// Copy exact native blob bytes into canonical caller-allocator string storage.
copy :: (data: *void, count: u64) -> string {
    require(count <= 0x7fff_ffff_ffff_ffff && (data != null || count == 0), "invalid native shader blob range");
    result := Basic.alloc_string(cast(s64) count);

    require(count == 0 || result.data != null, "shader blob copy allocation failed");
    if count != 0 memcpy(result.data, data, cast(s64) count);

    return result;
}

require :: (condition: bool, message: string) {
    if condition return;
    Basic.assert(false, message);
    abort();
}

#scope_file
Basic :: #import "Basic";

libc :: #library,system "c";
abort :: () #foreign libc;
