#scope_module
formatter_style :: () -> *Print_Style {
    return *context.print_style;
}

// The context callback has one canonical, assert-enabled builder ABI. Other
// Basic configurations retain their buffer layout and copy callback
// output through an owned canonical scratch builder instead of reinterpreting it.
#if !ENABLE_ASSERT Canonical_Print :: #import "internal/basic";

formatter_struct_callback :: (builder: *String_Builder, item: Any, style: *Print_Style) -> bool {
    if !style.struct_printer return false;

    #if ENABLE_ASSERT return style.struct_printer(builder, item, style.struct_printer_data);
    else {
        scratch: Shared_Print.String_Builder;
        scratch.allocator = builder.allocator;
        scratch.subsequent_buffer_size = builder.subsequent_buffer_size;

        // Preserve existing bytes so callbacks may inspect or replace the prefix.
        if builder.initialized {
            buffer := get_base_buffer(builder);
            while buffer {
                Canonical_Print.append(*scratch, get_buffer_data(buffer), buffer.count);
                buffer = buffer.next;
            }
        }

        scratch.failed = builder.failed;
        handled := style.struct_printer(*scratch, item, style.struct_printer_data);

        free_buffers(builder);
        builder.allocator = scratch.allocator;
        builder.subsequent_buffer_size = scratch.subsequent_buffer_size;

        if scratch.initialized {
            buffer := cast(*Shared_Print.String_Builder.Buffer) scratch.initial_bytes.data;
            while buffer {
                // Preserve output even when a callback requests default rendering.
                append(builder, cast(*u8) buffer + size_of(Shared_Print.String_Builder.Buffer), buffer.count);
                buffer = buffer.next;
            }
        }

        builder.failed = builder.failed || scratch.failed;
        Canonical_Print.free_buffers(*scratch);

        return handled;
    }
}

formatter_error :: (style: *Print_Style, message: string) {
    if style.log_runtime_errors context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
}

formatter_repeat :: (builder: *String_Builder, byte: u8, count: s64) {
    remaining := count;
    while remaining > 0 {
        range := ensure_contiguous_space_and_return_available_range(builder);

        if builder.failed return;

        amount := min(remaining, range.count);

        memset(range.data, byte, amount);
        advance_through_ensured_space(builder, amount);
        remaining -= amount;
    }
}
