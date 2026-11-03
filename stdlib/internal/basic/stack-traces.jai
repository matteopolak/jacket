// Copies the current stack trace into a slice that stays valid after the call returns. Empty when the program keeps no
// stack trace.
pack_stack_trace :: () -> [] Stack_Trace_Node #no_debug {
    if !context.stack_trace return .{};
    head := context.stack_trace.*;
    return pack_stack_trace(*head);
}

pack_stack_trace :: (node: *Stack_Trace_Node) -> [] Stack_Trace_Node {
    validate_trace(node);
    count: s64;
    current := node;

    while current {
        count += 1;
        current = current.next;
    }

    if count == 0 return .{};

    assert(count <= 0x7fffffffffffffff / size_of(Stack_Trace_Node));
    result := ([] Stack_Trace_Node).{ count, cast(*Stack_Trace_Node) alloc(count * size_of(Stack_Trace_Node)) };

    assert(result.data != null, "stack trace allocation failed");
    current = node;

    for index: 0..count - 1 {
        result[index] = current.*;
        result[index].next = ifx index + 1 < count then result.data + index + 1 else null;
        current = current.next;
    }

    return result;
}

// The frames of a stack trace (the current one by default) as strings, innermost first, in a dynamic array.
get_stack_trace_strings :: (node: *Stack_Trace_Node = null) -> [..] string #no_debug {
    head: Stack_Trace_Node;
    current := node;

    if !current && context.stack_trace {
        head = context.stack_trace.*;
        current = *head;
    }

    validate_trace(current);
    result: [..] string;
    result.allocator = temp;

    while current {
        if current.info array_add(*result, frame_string(current));
        current = current.next;
    }

    return result;
}

// Prints each frame of the trace at `node` on its own line, after `prefix`, to standard error unless `to_standard_error`
// is `false`.
//
// ```jai,no_run
// print_stack_trace(context.stack_trace, "  at ");
// ```
print_stack_trace :: (node: *Stack_Trace_Node, prefix: string = "", to_standard_error := true) {
    validate_trace(node);
    current := node;
    while current {
        if current.info {
            write_strings(prefix, "'", current.info.name, "' at ", current.info.location.fully_pathed_filename, ":", to_standard_error = to_standard_error);
            write_number(cast(s64) current.line_number, to_standard_error = to_standard_error);
            write_string("\n", to_standard_error = to_standard_error);
        }

        current = current.next;
    }
}

// Sends each frame of a stack trace (the current one by default) to the logger, one message per frame.
//
// ```jai,no_run
// log_stack_trace();
// ```
log_stack_trace :: (node: *Stack_Trace_Node = null, log_flags := Log_Flags.ERROR, prefix := "") #no_debug {
    head: Stack_Trace_Node;
    current := node;

    if !current && context.stack_trace {
        head = context.stack_trace.*;
        current = *head;
    }

    validate_trace(current);

    while current {
        if current.info log("%%\n", prefix, frame_string(current), flags = log_flags);
        current = current.next;
    }
}

#scope_module
validate_trace :: (node: *Stack_Trace_Node) {
    slow := node;
    fast := node;

    while fast && fast.next {
        slow = slow.next;
        fast = fast.next.next;
        assert(slow != fast, "a stack trace must not contain a cycle");
    }
}

frame_string :: (node: *Stack_Trace_Node) -> string {
    return tprint("'%' at %:%", node.info.name, node.info.location.fully_pathed_filename, node.line_number);
}
