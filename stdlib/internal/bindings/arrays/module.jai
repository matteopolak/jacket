// Context-owned arrays for native count/pointer convenience procedures.
allocate_values :: (count: s64, $T: Type) -> [] T, *void {
    if count == 0 return .[], null;

    bytes := count * size_of(T);
    base := context.allocator.proc(.ALLOCATE, bytes, 0, null, context.allocator.data);

    if !base return .[], null;

    data := cast(*T) base;

    for i: 0..count - 1 {
        initial: T;
        data[i] = initial;
    }

    return .{ data = data, count = count }, base;
}

release_values :: (memory: *void) {
    context.allocator.proc(.FREE, 0, 0, memory, context.allocator.data);
}
