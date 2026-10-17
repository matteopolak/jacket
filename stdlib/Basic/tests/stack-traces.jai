#import "Basic";
#import "String";

calls: int;
message: string;
received: Log_Info;

capture :: (text: string, data: *void, info: Log_Info) {
    calls += 1;
    free(message);
    message = copy_string(text);
    received = info;
}

main :: () {
    first_info := Stack_Trace_Procedure_Info.{ name = "inner", location = .{ fully_pathed_filename = "inside.jai", line_number = 7 } };
    second_info := Stack_Trace_Procedure_Info.{ name = "outer", location = .{ fully_pathed_filename = "outside.jai", line_number = 21 } };
    second := Stack_Trace_Node.{ info = *second_info, line_number = 29, hash = 2, call_depth = 1 };
    first := Stack_Trace_Node.{ info = *first_info, line_number = 11, hash = 3, call_depth = 2, next = *second };
    packed := pack_stack_trace(*first);

    defer array_free(packed);
    assert(packed.count == 2 && packed.data != *first);
    assert(packed[0].next == packed.data + 1 && packed[1].next == null);
    assert(packed[0].info == *first_info && packed[1].info == *second_info);
    assert(packed[0].hash == 3 && packed[0].line_number == 11 && packed[0].call_depth == 2);
    assert(pack_stack_trace(null).count == 0);

    backing: [4096] u8;
    storage: Temporary_Storage_Paged_Alloc;
    storage.data = backing.data;
    storage.size = backing.count;

    push_context {
        context.temporary_storage = *storage;
        context.stack_trace = *first;
        context.logger = capture;
        context.log_source_identifier = 177;
        implicit := pack_stack_trace();

        defer array_free(implicit);
        assert(implicit.count == 2 && implicit[0].info == *first_info);
        strings := get_stack_trace_strings(packed.data);

        assert(strings.count == 2);
        assert(strings.allocator.proc == temp.proc);
        assert(cast(*u8) strings.data >= backing.data && cast(*u8) strings.data < backing.data + backing.count);
        assert(strings[0].data >= backing.data && strings[0].data < backing.data + backing.count);
        assert(contains(strings[0], "inner") && contains(strings[0], "inside.jai") && contains(strings[0], "11"));
        assert(contains(strings[1], "outer") && contains(strings[1], "outside.jai") && contains(strings[1], "29"));
        log_stack_trace(log_flags = .WARNING, prefix = "trace: ");
        assert(calls == 2 && contains(message, "trace: ") && contains(message, "outer"));
        assert(received.common_flags == .WARNING && received.source_identifier == 177);
        assert(contains(message, "outside.jai:29\n"));
        unknown := Stack_Trace_Node.{ next = packed.data };
        with_unknown := get_stack_trace_strings(*unknown);

        assert(with_unknown.count == 2 && with_unknown[0] == strings[0]);
    }

    free(message);
    print("ok\n");
}
