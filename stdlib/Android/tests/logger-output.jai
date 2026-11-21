// test: only(.LINUX,.MACOS)
// Independent host control for the real shared Runtime queues and logger core.
// The sink writes actual host descriptors; this is not an Android SDK mock.
Log :: #import "internal/bindings/android-log";
Output :: #import "internal/runtime/android-output";
SameOutput :: #import "internal/runtime/android-output";
#import "Basic";
P :: #import "POSIX";

libc :: #library,system "c";
native_write :: (fd: s32, data: *u8, count: P.size_t) -> P.ssize_t #foreign libc "write"; @WritesOutput(0)
abort :: () #foreign libc; @Exits

emit :: (text: string) #no_context {
    offset: s64;
    while offset < text.count {
        written := native_write(1, text.data + offset, cast(P.size_t) (text.count - offset));
        if written <= 0 abort();
        offset += cast(s64) written;
    }
}

view :: (text: *u8) -> string #no_context {
    count: s64;
    while text[count] != 0 count += 1;
    return .{ data = text, count = count };
}

host_sink :: (priority: s32, tag: *u8, text: *u8) -> s32 #c_call {
    digit := cast(u8) (priority + #char "0");

    emit(.{ data = *digit, count = 1 });
    emit(":");
    emit(view(tag));
    emit(":");
    emit(view(text));
    emit("\n");
    // Return the actual native write result; no claimed Android SDK status.
    marker := "sink-complete\n";

    return cast(s32) native_write(1, marker.data, cast(P.size_t) marker.count);
}

main :: () {
    allocator := context.allocator;
    info: Log_Info;

    Output.append("STALE-STDOUT");
    SameOutput.append("STALE-STDERR", true);
    Log.write("payload", info, host_sink);
    Output.flush(true);
    Output.append("fresh-out\n");
    SameOutput.append("fresh-error\n", true);
    Output.append("SECOND-STALE");
    Output.flush(false);
    section: Log_Section;
    section.name = "tag";
    info.section = *section;
    info.common_flags = .ERROR | .WARNING;
    Log.write("second", info, host_sink);
    SameOutput.flush(true);
    if context.allocator.proc != allocator.proc || context.allocator.data != allocator.data abort();
}
