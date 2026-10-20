#scope_module
raw_trace_copy :: (allocator: Allocator, node: *Stack_Trace_Node) -> *Packed_Stack_Trace {
    validate_trace(node);
    count: s64;
    current := node;

    while current {
        count += 1;
        current = current.next;
    }

    bytes := size_of(Packed_Stack_Trace) + count * size_of(Stack_Trace_Node);
    result := cast(*Packed_Stack_Trace) allocator.proc(.ALLOCATE, bytes, 0, null, allocator.data);

    if !result {
        context.assertion_failed(#caller_location, "memory debugger: trace allocation failed");
        debug_break();
    }

    result.nodes = .{ count, cast(*Stack_Trace_Node) (cast(*u8) result + size_of(Packed_Stack_Trace)) };
    current = node;

    for index: 0..count - 1 {
        result.nodes[index] = current.*;
        result.nodes[index].next = ifx index + 1 < count then result.nodes.data + index + 1 else null;
        current = current.next;
    }

    return result;
}

raw_trace_free :: (allocator: Allocator, trace: *Packed_Stack_Trace) {
    if !trace return;
    allocator.proc(.FREE, 0, size_of(Packed_Stack_Trace) + trace.nodes.count * size_of(Stack_Trace_Node), trace, allocator.data);
}
