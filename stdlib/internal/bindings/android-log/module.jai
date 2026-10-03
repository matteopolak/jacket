// Android facade and host controls share this core. Runtime owns the queues;
// the platform supplies only its actual C log sink, never another buffer.
Sink :: #type (priority: s32, tag: *u8, message: *u8) -> s32 #c_call;

write :: (message: string, info: Log_Info, sink: Sink) {
    tag := Native.terminated_copy(Policy.log_tag(info));

    defer Native.release(tag);
    payload := Native.terminated_copy(message);

    defer Native.release(payload);
    Runtime.discard_pending_output();
    sink(Policy.log_priority(info.common_flags), tag, payload);
}

#scope_file
Runtime :: #import "internal/runtime/android-output";
Native :: #import "internal/bindings/native";
Policy :: #import "internal/bindings/android-policy";
