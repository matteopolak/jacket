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

    push_context {
        context.stack_trace = *first;
        context.logger = capture;
        context.log_source_identifier = 177;
        implicit := pack_stack_trace();

        defer array_free(implicit);
        assert(implicit.count == 2 && implicit[0].info == *first_info);
        strings := get_stack_trace_strings(packed.data);

        defer {
            for strings free(it);
            array_free(strings);
        }

        assert(strings.count == 2);
        assert(contains(strings[0], "inner") && contains(strings[0], "inside.jai") && contains(strings[0], "11"));
        assert(contains(strings[1], "outer") && contains(strings[1], "outside.jai") && contains(strings[1], "29"));
        log_stack_trace(log_flags = .WARNING, prefix = "trace: ");
        assert(calls == 2 && contains(message, "trace: ") && contains(message, "outer"));
        assert(received.common_flags == .WARNING && received.source_identifier == 177);
        assert(received.location.fully_pathed_filename == "outside.jai" && received.location.line_number == 29);
    }

    free(message);
    print("ok\n");
}
