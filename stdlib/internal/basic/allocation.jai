alloc :: (size: s64) -> *void {
    assert(size >= 0);
    return allocator_call(context.allocator, .ALLOCATE, size, 0, null);
}

get_capabilities :: (b: Allocator) -> (caps: Allocator_Caps, name: string) {
    name: string;
    flags := b.proc(.CAPS, 0, 0, *name, null);
    return cast(Allocator_Caps) cast(u64) flags, name;
}

free :: (memory: *void) {
    allocator_call(context.allocator, .FREE, 0, 0, memory);
} @Releases

realloc :: (memory: *void, size: s64, old_size: s64) -> *void {
    assert(size >= 0 && old_size >= 0);
    return allocator_call(context.allocator, .RESIZE, size, old_size, memory);
}

free :: inline (s: string) {
    free(s.data);
} @Releases

alloc_string :: (count: int) -> string {
    memory := alloc(count);
    assert(memory != null || count == 0);
    return .{ count, cast(*u8) memory };
}

copy_string :: (s: string) -> string {
    result := alloc_string(s.count);
    memcpy(result.data, s.data, s.count);
    return result;
} @UseResult

to_string :: inline (s: *u8, length: s64) -> string {
    return .{ length, s };
}

to_string :: (c_string: [] u8) -> string {
    i: s64 = 0;
    while i < c_string.count && c_string[i] != 0 i += 1;
    return .{ i, c_string.data };
}

to_c_string :: (s: string) -> *u8 {
    result := cast(*u8) alloc(s.count + 1);

    assert(result != null);
    memcpy(result, s.data, s.count);
    result[s.count] = 0;

    return result;
}

Initialize :: (memory: *$T) {
    memory.* = T.{};
}

New :: ($T: Type, $initialized := true) -> *T {
    memory := cast(*T) alloc(size_of(T));

    assert(memory != null);
    assert(cast(u64) memory % align_of(T) == 0, "allocator returned storage with insufficient alignment");
    #if initialized Initialize(memory);

    return memory;
}

NewArray :: (count: s64, $T: Type, $initialized := true, alignment: s32 = -1) -> (result: [] T, unaligned_base: *void) {
    assert(count >= 0);
    desired := cast(s64) alignment;
    if desired < 0 desired = align_of(T);

    assert(desired >= align_of(T) && (desired & (desired - 1)) == 0);
    if count == 0 return .{}, null;

    #if size_of(T) > 0 assert(count <= (0x7fffffffffffffff - desired) / size_of(T));
    base := alloc(count * size_of(T) + desired - 1);

    assert(base != null);
    data := cast(*T) align_forward(cast(s64) base, desired);
    result: [] T = .{ count, data };

    #if initialized for *value: result Initialize(value);

    return result, base;
}

align_forward :: (orig_size: s64, alignment: s64) -> s64 {
    assert(alignment > 0);
    return ((orig_size + alignment - 1) / alignment) * alignment;
}

push_allocator :: (allocator: Allocator) #expand #no_debug {
    previous_allocator := context.allocator;
    context.allocator = allocator;
    `defer context.allocator = previous_allocator;
}

push_allocator :: (proc: Allocator_Proc, data: *void) #expand #no_debug {
    previous_allocator := context.allocator;
    context.allocator = Allocator.{ proc, data };
    `defer context.allocator = previous_allocator;
}
