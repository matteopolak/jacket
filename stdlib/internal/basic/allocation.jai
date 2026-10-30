alloc :: (size: s64) -> *void {
    assert(size >= 0);
    return allocator_call(context.allocator, .ALLOCATE, size, 0, null);
} @Allocates

get_capabilities :: (b: Allocator) -> (caps: Allocator_Caps, name: string) {
    name: string;
    flags := b.proc(.CAPS, 0, 0, *name, null);
    return cast(Allocator_Caps) cast(u64) flags, name;
}

// Gives memory back to the context allocator. `memory` must come from `alloc`, `New`, `realloc` or another
// procedure that allocates with the context allocator, and must not be freed twice. Freeing a null
// pointer is up to the allocator; do not rely on it.
free :: (memory: *void) {
    allocator_call(context.allocator, .FREE, 0, 0, memory);
} @FreesArgument

realloc :: (memory: *void, size: s64, old_size: s64) -> *void {
    assert(size >= 0 && old_size >= 0);
    return allocator_call(context.allocator, .RESIZE, size, old_size, memory);
}

// Frees the memory a string points to. The string must have been allocated with the context allocator
// in effect now, for example by `alloc_string`, `copy_string` or `sprint`. Never free a string literal,
// a slice of another string or a string from temporary storage.
free :: inline (s: string) {
    free(s.data);
} @FreesArgument

// A string of `count` bytes in newly allocated memory, with the context allocator. The bytes are not
// cleared, so write all of them before reading. Free it with `free`. With a `count` of 0 the string is
// empty and may have no data.
//
// ```jai
// s := alloc_string(3);
// defer free(s);
// memcpy(s.data, "abc".data, 3);
// assert(s == "abc");
// ```
alloc_string :: (count: int) -> string {
    memory := alloc(count);
    assert(memory != null || count == 0);
    return .{ count, cast(*u8) memory };
} @Allocates

// A copy of `s` in newly allocated memory, with the context allocator. Free it with `free`. Use
// `copy_temporary_string` for a copy that needs no freeing.
//
// ```jai
// original := "text";
// copy := copy_string(original);
// defer free(copy);
// assert(copy == original && copy.data != original.data);
// ```
copy_string :: (s: string) -> string {
    result := alloc_string(s.count);
    memcpy(result.data, s.data, s.count);
    return result;
} @UseResult @Allocates

// A string that views `length` bytes at `s`, without copying them. Nothing is allocated, so the string
// is valid as long as the bytes are.
//
// ```jai
// bytes := "hello".data;
// view := to_string(bytes, 4);
// assert(view == "hell");
// ```
to_string :: inline (s: *u8, length: s64) -> string {
    return .{ length, s };
}

// A string that views the bytes of `c_string` up to the first zero byte, or all of them when there is none.
// The bytes are not copied.
//
// ```jai
// buffer: [8] u8;
// buffer[0] = #char "h";
// buffer[1] = #char "i";
// assert(to_string(buffer) == "hi");
// ```
to_string :: (c_string: [] u8) -> string {
    i: s64 = 0;
    while i < c_string.count && c_string[i] != 0 i += 1;
    return .{ i, c_string.data };
}

// A copy of `s` in newly allocated memory, followed by a zero byte, for passing to C. It is allocated with
// the context allocator; free it with `free`. A zero byte inside `s` ends the string for the C side.
//
// ```jai
// c := to_c_string("abc");
// defer free(c);
// assert(c[3] == 0 && to_string(c, 3) == "abc");
// ```
to_c_string :: (s: string) -> *u8 {
    result := cast(*u8) alloc(s.count + 1);

    assert(result != null);
    memcpy(result, s.data, s.count);
    result[s.count] = 0;

    return result;
} @Allocates

Initialize :: (memory: *$T) {
    memory.* = T.{};
}

// Allocates a `T` with `context.allocator` and returns a pointer to it.
//
// - initialized: set to `false` to skip setting the new value to its default; its bytes are then
//   whatever the allocator returned.
New :: ($T: Type, $initialized := true) -> *T {
    memory := cast(*T) alloc(size_of(T));

    assert(memory != null);
    assert(cast(u64) memory % align_of(T) == 0, "allocator returned storage with insufficient alignment");
    #if initialized Initialize(memory);

    return memory;
} @Allocates

// Allocates `count` elements of `T` with `context.allocator`.
//
// The second result is the pointer the allocator gave out. Free that one, not `result.data`, which may have been moved forward to meet `alignment`.
//
// - alignment: in bytes, a power of two. The type's own alignment when negative.
// - initialized: set to `false` to leave the elements as the allocator returned them.
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
} @Allocates

align_forward :: (orig_size: s64, alignment: s64) -> s64 {
    assert(alignment > 0);
    return ((orig_size + alignment - 1) / alignment) * alignment;
}

push_allocator :: (allocator: Allocator) #expand #no_debug {
    previous_allocator := context.allocator;
    context.allocator = allocator;
    `defer context.allocator = previous_allocator;
} @PushesContext

// Makes `Allocator.{proc, data}` the `context.allocator` until the end of the enclosing scope.
//
// - proc: the allocator procedure.
// - data: handed to `proc` on every call.
push_allocator :: (proc: Allocator_Proc, data: *void) #expand #no_debug {
    previous_allocator := context.allocator;
    context.allocator = Allocator.{ proc, data };
    `defer context.allocator = previous_allocator;
} @PushesContext
