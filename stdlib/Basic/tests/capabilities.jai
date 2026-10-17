#import "Basic";

allocator_name :: "borrowed allocator";
calls: int;

named_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    assert(mode == .CAPS && size == 0 && old_size == 0 && old_memory != null && allocator_data == null);
    calls += 1;
    (cast(*string) old_memory).* = allocator_name;
    return cast(*void) cast(u64) (Allocator_Caps.FREE | .IS_THIS_YOURS);
}

main :: () {
    flags, name := get_capabilities(.{ named_allocator, cast(*void) 1 });

    assert(calls == 1 && flags == (.FREE | .IS_THIS_YOURS));
    assert(name == allocator_name && name.data == allocator_name.data);
    temp_flags, temp_name := get_capabilities(temp);

    assert(temp_flags == .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE);
    assert(temp_name.count > 0);
    runtime_flags, runtime_name := get_capabilities(context.default_allocator);

    assert((runtime_flags & .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR) != 0);
    assert(runtime_name.count > 0);
    repeated_flags, repeated_name := get_capabilities(context.default_allocator);

    assert(repeated_flags == runtime_flags && repeated_name.data == runtime_name.data && repeated_name == runtime_name);
    print("ok\n");
}
