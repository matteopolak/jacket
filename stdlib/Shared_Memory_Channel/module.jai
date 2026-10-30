// Shared_Memory_Channel: a one-way message queue between two programs (or two parts of one) over shared memory.
//
// A channel has one reader and one writer. The reader creates it by name with `reader_create_channel`, the writer attaches
// with `writer_connect`. The writer sends a message by reserving room (`writer_reserve_message`), filling it in place and
// committing it (`writer_commit_message`); the reader finds messages in order with `reader_poll_for_message`, reads them
// in place and finishes each with `reader_done_with_message`. Nothing copies the payload and nothing blocks: a full ring
// makes a reserve fail, an empty one makes a poll find nothing, and the program decides how to wait. The ring keeps messages
// whole, 8-byte aligned, so a message must fit in the ring in one piece.
//
// The pieces are lock-free and safe for exactly one reader and one writer, possibly in different processes. The other end
// is not trusted to be well-behaved: a damaged header or cursor is reported as a fatal error instead of being followed.
// Needs Linux, macOS or Windows. Names and the lifetime of the shared object are described on `reader_create_channel`.
//
// ```jai,skip_on(wasm)
// name := tprint("jai-doc-module-%", to_nanoseconds(current_time_monotonic()));
//
// ok, reader := reader_create_channel(name, 512);
// assert(ok);
// connected, writer := writer_connect(name);
// assert(connected);
//
// reserved_ok, reserved := writer_reserve_message(*writer, 3);
// assert(reserved_ok);
// memcpy(reserved.user_write_pointer, "abc".data, 3);
// assert(writer_commit_message(*writer, reserved));
//
// found, fatal, message := reader_poll_for_message(*reader);
// assert(found && !fatal && message.user_read_bytes == 3);
// assert(reader_done_with_message(*message));
//
// assert(writer_disconnect(*writer));
// assert(reader_release_channel(*reader));
// ```

#assert OS == .MACOS || OS == .LINUX || OS == .WINDOWS, "Shared_Memory_Channel requires native shared-memory mappings";

#load "../internal/ipc/channel-types.jai";
#load "../internal/ipc/channel.jai";
#if OS == .WINDOWS #load "../internal/ipc/channel-windows.jai";
else #load "../internal/ipc/channel-posix.jai";
