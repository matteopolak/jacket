// Logging through a custom context.logger (flags, sections, locations, return values), assertion
// handling through context.assertion_failed, stack-trace helpers on a hand-built trace,
// String_Builder across many segments and with a failing allocator, and format-string edge cases.
#import "Basic";

Captured :: struct {
    message: string;
    info: Log_Info;
}

captured: [..] Captured;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(data == *captured, "logger_data reaches the logger");
    array_add(*captured, .{ copy_string(message), info });
}

Assertion :: struct {
    location: Source_Code_Location;
    message: string;
}

assertions: [..] Assertion;

record_assertion :: (location: Source_Code_Location, message: string) -> bool {
    array_add(*assertions, .{ location, copy_string(message) });
    // A failure inside the handler is swallowed, not recursed into.
    assert(false, "nested failure");
    return false;
}

main :: () {
    check_logging();
    check_assertions();
    check_stack_traces();
    check_builder();
    check_failing_builder();
    check_formatting();
    print("ok\n");
}

check_logging :: () {
    scoped := context;
    push_context, defer_pop scoped;
    context.logger = capture;
    context.logger_data = *captured;
    context.log_source_identifier = 4242;

    line_before := #location().line_number;
    count := log("plain % %", 1, "two");
    assert(count == "plain 1 two".count, "log returns the length: %", count);
    log_error("bad %", 3.5);
    log_warning("careful");
    section := Log_Section.{ name = "net" };
    log(*section, "in a section %", 9, flags = .VERBOSE_ONLY, user_flags = 77);
    log_error(*section, "section error");
    log("with section argument", section = *section, user_flags = 5);
    two_slots := "% %";
    assert(log(two_slots, 1) == -1, "a missing argument fails the log");

    assert(captured.count == 7, "% messages", captured.count);
    c := captured;
    assert(c[0].message == "plain 1 two" && c[0].info.common_flags == .NONE);
    assert(c[0].info.source_identifier == 4242, "source identifier %", c[0].info.source_identifier);
    assert(c[0].info.location.line_number == line_before + 1, "logged from line % (expected %)", c[0].info.location.line_number, line_before + 1);
    assert(ends_with(c[0].info.location.fully_pathed_filename, "basic-logging-builder.jai"), "file %", c[0].info.location.fully_pathed_filename);
    assert(c[1].message == "bad 3.5" && c[1].info.common_flags == .ERROR, "% %", c[1].message, c[1].info.common_flags);
    assert(c[2].message == "careful" && c[2].info.common_flags == .WARNING);
    assert(c[3].message == "in a section 9" && c[3].info.section == *section && c[3].info.common_flags == .VERBOSE_ONLY && c[3].info.user_flags == 77);
    assert(c[4].message == "section error" && c[4].info.common_flags == .ERROR && c[4].info.section.name == "net");
    assert(c[5].info.section == *section && c[5].info.user_flags == 5);
    assert(c[6].message == "1 <missing format argument>", "a failed format still logs: %", c[6].message);
    for captured free(it.message);
    array_reset(*captured);

    // A logger set inside a nested context does not leak out of it.
    {
        scoped := context;
        push_context, defer_pop scoped;
        context.logger = (message: string, data: *void, info: Log_Info) {
            array_add(*captured, .{ "inner", info });
        };
        log("ignored text");
    }
    log("outer");
    assert(captured.count == 2 && captured[0].message == "inner" && captured[1].message == "outer", "captured %", captured);
    array_reset(*captured);
}

check_assertions :: () {
    scoped := context;
    push_context, defer_pop scoped;
    context.assertion_failed = record_assertion;
    line := #location().line_number;
    assert(1 + 1 == 3, "arithmetic is % (%)", "broken", 42);
    assert(true, "never reported");
    assert(false);
    assert(assertions.count == 2, "% assertions reported", assertions.count);
    assert(assertions[0].message == "arithmetic is broken (42)", "message %", assertions[0].message);
    assert(assertions[0].location.line_number == line + 1, "assertion line % (expected %)", assertions[0].location.line_number, line + 1);
    assert(assertions[1].message == "");
    assert(!context.handling_assertion_failure, "the handler flag is cleared afterwards");
    // assert_helper reports the handler's answer.
    assert(!assert_helper("direct %", 1));
    assert(assertions.count == 3 && assertions[2].message == "direct 1");
}

check_stack_traces :: () {
    infos: [3] Stack_Trace_Procedure_Info;
    infos[0] = .{ name = "inner", location = .{ "/src/a.jai", 10, 1 } };
    infos[1] = .{ name = "middle", location = .{ "/src/b.jai", 20, 1 } };
    infos[2] = .{ name = "outer", location = .{ "/src/c.jai", 30, 1 } };
    nodes: [3] Stack_Trace_Node;
    for *nodes {
        it.info = *infos[it_index];
        it.line_number = cast(u32) (it_index + 1) * 11;
        it.call_depth = cast(u32) (3 - it_index);
        if it_index < 2 it.next = *nodes[it_index + 1];
    }
    packed := pack_stack_trace(*nodes[0]);
    assert(packed.count == 3, "packed % frames", packed.count);
    for packed {
        assert(it.info == *infos[it_index] && it.line_number == nodes[it_index].line_number);
        if it_index < 2 assert(it.next == *packed[it_index + 1], "packed frames link to each other");
        else assert(it.next == null);
    }
    free(packed.data);
    assert(pack_stack_trace(null).count == 0);
    text := get_stack_trace_string(*nodes[0]);
    assert(text == "/src/a.jai:11 inner\n/src/b.jai:22 middle\n/src/c.jai:33 outer\n", "trace text %", text);
    free(text);
    assert(get_stack_trace_string(null) == "");

    scoped := context;
    push_context, defer_pop scoped;
    context.logger = capture;
    context.logger_data = *captured;
    log_stack_trace(*nodes[1], prefix = "trace:\n");
    assert(captured.count == 1 && captured[0].message == "trace:\n/src/b.jai:22 middle\n/src/c.jai:33 outer\n", "logged %", captured[0].message);
    assert(captured[0].info.common_flags == .ERROR);
    array_reset(*captured);
    // The live trace, when this build keeps one.
    if context.stack_trace {
        live := pack_stack_trace();
        // pack_stack_trace's own frames come first, then this procedure called from main.
        found := -1;
        for live if it.info && it.info.name == "check_stack_traces" {
            found = it_index;
            break;
        }
        assert(found >= 0, "check_stack_traces is on the live trace");
        assert(found + 1 < live.count && live[found + 1].info.name == "main", "called from main");
        free(live.data);
    }
}

check_builder :: () {
    builder: String_Builder;
    assert(builder_string_length(*builder) == 0, "an untouched builder is empty");
    assert(builder_to_string(*builder) == "");
    make_builder(*builder);
    // Enough text to need many segments, written in uneven pieces.
    expected: [..] u8;
    defer array_free(expected);
    for i: 0..2999 {
        piece := tprint("[%:%]", i, i * i);
        append(*builder, piece);
        array_add(*expected, ..cast([] u8) piece);
        if i % 7 == 0 {
            append(*builder, cast(u8) #char "|");
            array_add(*expected, #char "|");
        }
    }
    assert(builder_string_length(*builder) == expected.count, "length % vs %", builder_string_length(*builder), expected.count);
    assert(builder.current_buffer != null, "the text spilled out of the inline buffer");
    with_prefix := builder_to_string(*builder, extra_bytes_to_prepend = 3, do_reset = false);
    assert(with_prefix.count == expected.count + 3 && with_prefix[0] == 0 && with_prefix[2] == 0);
    assert(string.{ expected.count, with_prefix.data + 3 } == cast(string) expected, "multi-segment contents");
    free(with_prefix);
    whole := builder_to_string(*builder);
    assert(whole == cast(string) expected);
    free(whole);
    assert(builder_string_length(*builder) == 0 && builder.current_buffer == null, "builder_to_string resets by default");

    // Raw values and ensured space.
    value: u32 = 0x64636261;
    append_by_pointer(*builder, *value);
    simple_memcpy(*builder, cast(u16) 0x6665);
    space := cast(*u8) ensure_contiguous_space(*builder, 4);
    assert(space != null);
    memcpy(space, "wxyz".data, 4);
    advance_through_ensured_space(*builder, 4);
    range := ensure_contiguous_space_and_return_available_range(*builder, 1);
    assert(range.count > 0);
    assert(ensure_contiguous_space_and_return_available_range(*builder, -1).count == 0);
    small_endian := builder_to_string(*builder, do_reset = false);
    #if CPU == .X64 || CPU == .ARM64 || OS == .WASM assert(small_endian == "abcdefwxyz", "raw bytes %", small_endian);
    free(small_endian);

    // Ensured space at a segment's end moves to a fresh segment.
    sized: String_Builder;
    init_string_builder(*sized, 64);
    available := ensure_contiguous_space_and_return_available_range(*sized, 1);
    append(*sized, "x");
    for 1..available.count - 1 append(*sized, "y");
    assert(sized.current_buffer == null, "the inline buffer is exactly full");
    p := ensure_contiguous_space(*sized, 10);
    assert(p != null && sized.current_buffer != null && sized.current_buffer.allocated == 64, "a 64-byte spill segment");
    assert(ensure_contiguous_space(*sized, 65) == null, "more than a segment can hold");
    write_count := builder_string_length(*sized);
    assert(write_count == available.count);
    free(*sized);
    assert(builder_string_length(*sized) == 0);

    // Stealing another builder's text.
    other: String_Builder;
    for 1..1000 append(*other, "0123456789");
    append(*builder, "!");
    append_and_steal_buffers(*builder, *other);
    assert(builder_string_length(*other) == 0, "the source is emptied");
    stolen := builder_to_string(*builder);
    assert(stolen.count == 10 + 1 + 10000 && stolen[10] == #char "!" && stolen[11] == #char "0" && stolen[stolen.count - 1] == #char "9", "stolen %", stolen.count);
    free(stolen);
    append_and_steal_buffers(*builder, *builder);
    reset(*other);
    free_buffers(*other);

    // consume_u8_and_length.
    input := "prefix:rest";
    assert(!consume_u8_and_length(*input, "prefiz".data, 6) && input == "prefix:rest");
    assert(consume_u8_and_length(*input, "prefix:".data, 7) && input == "rest");
    assert(!consume_u8_and_length(*input, "restless".data, 8), "longer than the input");
    assert(!consume_u8_and_length(*input, "r".data, -1));
    assert(consume_u8_and_length(*input, "".data, 0) && input == "rest");

    // write_builder writes and counts every segment.
    out: String_Builder;
    append(*out, "write_builder line\n");
    assert(write_builder(*out) == 19);
    assert(builder_string_length(*out) == 0);
    reset(*builder);
}

Failing :: struct {
    budget: s64;
}

failing_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    state := cast(*Failing) data;
    if mode == .ALLOCATE || mode == .RESIZE {
        if state.budget <= 0 return null;
        state.budget -= 1;
    }
    return Context_Base.default_allocator.proc(mode, size, old_size, old_memory, Context_Base.default_allocator.data);
}

check_failing_builder :: () {
    state := Failing.{ budget = 1 };
    push_allocator(failing_proc, *state);
    builder: String_Builder;
    init_string_builder(*builder);
    // One spill segment is allowed, the next is refused.
    for 1..3 for 1..STRING_BUILDER_BUFFER_SIZE append(*builder, "z");
    assert(builder.failed, "running out of memory marks the builder failed");
    kept := builder_string_length(*builder);
    assert(kept > STRING_BUILDER_BUFFER_SIZE && kept < 3 * STRING_BUILDER_BUFFER_SIZE, "kept %", kept);
    assert(ensure_contiguous_space(*builder, 100) == null);
    state.budget = 10;
    reset(*builder);
    assert(!builder.failed, "reset clears the failure");
    free_buffers(*builder);
}

Point :: struct {
    x: s32;
    y: s32;
}

check_formatting :: () {
    builder: String_Builder;
    assert(print_to_builder(*builder, "%2 %1 %", "a", "b"));
    assert(builder_to_string(*builder) == "b a b", "%N then %");
    third := "%3";
    assert(!print_to_builder(*builder, third, 1));
    assert(builder_to_string(*builder) == "<missing format argument>");
    assert(print_to_builder(*builder, "x%00y"));
    assert(builder_to_string(*builder) == "xy", "%00 prints nothing");
    assert(print_to_builder(*builder, "100\% %", 5));
    assert(builder_to_string(*builder) == "100% 5", "an escaped percent");
    assert(print_to_builder(*builder, "%%", 1, 2));
    assert(builder_to_string(*builder) == "12");
    assert(print_to_builder(*builder, "no arguments"));
    assert(builder_to_string(*builder) == "no arguments");
    assert(print_to_builder(*builder, ""));
    assert(builder_to_string(*builder) == "");

    print_sequence_to_builder(*builder, 1, "-", 2.5, "-", true);
    assert(builder_to_string(*builder) == "1-2.5-true");

    point := Point.{ 3, -4 };
    append_struct_info(*builder, type_info(Point), *point);
    assert(builder_to_string(*builder) == "{3, -4}", "struct %", builder_to_string(*builder, do_reset = false));
    format := FormatStruct.{ use_long_form_if_more_than_this_many_members = 0, begin_string = "<", end_string = ">" };
    append_struct_info(*builder, type_info(Point), *point, *format);
    long := builder_to_string(*builder);
    assert(long == "<x = 3; y = -4; >", "long form %", long);

    numbers := s64.[1, 2, 3, 4, 5];
    any: Any = numbers;
    print_array(*builder, any, cast(*Type_Info_Array) type_info([5] s64), .{ stop_printing_after_this_many_elements = 2 });
    assert(builder_to_string(*builder) == "[1, 2, ...]", "a truncated array");
    print_array(*builder, any, cast(*Type_Info_Array) type_info([5] s64), .{ separator = "|", begin_string = "", end_string = "", draw_separator_after_last_element = true });
    assert(builder_to_string(*builder) == "1|2|3|4|5|", "custom separators");
    view: [] s64;
    print_item_to_builder(*builder, view);
    assert(builder_to_string(*builder) == "[]");

    assert(sprint("%", FormatInt.{ value = 255, base = 16, minimum_digits = 4 }) == "00ff");
    assert(tprint("%", FormatInt.{ value = 1234567, digits_per_comma = 3, comma_string = "," }) == "1,234,567");
    assert(tprint("%", FormatInt.{ value = -5, minimum_digits = 3 }) == "-005", "%", tprint("%", FormatInt.{ value = -5, minimum_digits = 3 }));
    assert(tprint("%", cast(u64) 0xffff_ffff_ffff_ffff) == "18446744073709551615");
    assert(tprint("%", -9223372036854775807 - 1) == "-9223372036854775808");
    assert(tprint("%", cast(s8) -128) == "-128");

    end, value := parse_number("ab123cd", 2);
    assert(end == 5 && value == 123, "parse_number % %", end, value);
    end, value = parse_number("99999999999999999999", 0);
    assert(value == 999999999999999999 && end == 18, "parse_number stops before overflow: % %", end, value);
    end, value = parse_number("x", 1);
    assert(end == 1 && value == 0);
    assert(my_is_digit(#char "0") && my_is_digit(#char "9") && !my_is_digit(#char "a") && !my_is_digit(#char "/"));

    // print returns the bytes written, or -1 when the format fails.
    assert(print("print %\n", "line") == 11);
    two_lines := "% %\n";
    assert(print(two_lines, "missing") == -1);
    print_character(#char "c");
    print_character(#char "\n");
    bytes := u8.[0x41, 0x42];
    print_bytes(bytes.data, 2, "<%>");
    print_character(#char "\n");
}

ends_with :: (s: string, suffix: string) -> bool {
    if s.count < suffix.count return false;
    return string.{ suffix.count, s.data + s.count - suffix.count } == suffix;
}
