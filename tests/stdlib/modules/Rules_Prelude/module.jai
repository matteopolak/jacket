// A module with no imports of its own: Preload and Runtime_Support names are still visible.
// Used by tests/stdlib/rules-prelude-and-runtime-support.jai.

struct_member_count :: (T: Type) -> int {
    info := cast(*Type_Info) T;
    if info.type != .STRUCT return -1;
    return (cast(*Type_Info_Struct) info).members.count;
}

copy_bytes :: (dest: *void, source: *void, count: s64) {
    memcpy(dest, source, count);
}

allocator_is_set :: () -> bool {
    a: Allocator = context.allocator;
    return a.proc != null;
}
