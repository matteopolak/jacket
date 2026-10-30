PROTOCOL_VERSION :: 1;
CHANNEL_MAGIC :: "ShmC";
MESSAGE_MAGIC :: "ShmM2022";
WRAPPING_SENTINEL :: 0xcafebabef00dface;
OFFSET_TO_HEADER_OS_SPECIFIC :: 64;
OFFSET_TO_MESSAGE_QUEUE_HEADER :: 128;
OFFSET_TO_MESSAGE_DATA :: 256;
PER_MESSAGE_OVERHEAD_SIZE :: 16;

// The first record of the shared memory region: it identifies the channel and says how the rest is laid out.
//
// `magic` and `version` mark a valid channel (checked on every call; a region that fails the check reads as corrupt).
// `reader_status` and `writer_status` say whether each end is `DISCONNECTED`, `PAUSED` or `ACTIVELY_*`: a new channel has an
// active reader and a disconnected writer. The offsets locate the OS-specific header, the queue cursors and the message
// data inside the region. `full_memory_region_size_in_bytes` is the size of the whole mapping and `data_size_in_bytes` the
// capacity of the message ring, always a multiple of 8. The fields are shared with the other process: read them, but do not write them.
Header :: struct {
    magic: u32;
    version: u32;
    reader_status: enum s32 {
        DISCONNECTED :: 0;
        PAUSED :: 1;
        ACTIVELY_READING :: 2;
    };
    writer_status: enum s32 {
        DISCONNECTED :: 0;
        PAUSED :: 1;
        ACTIVELY_WRITING :: 2;
    };
    offset_to_os_specific_header: u32;
    offset_to_message_queue_header: u32;
    offset_to_message_data: u32;
    full_memory_region_size_in_bytes: s64;
    data_size_in_bytes: s64;
}

// The second record of the region, at a fixed offset after the `Header`; it holds the channel marker the other end checks.
Header_Os_Specific :: struct {
    magic: u32;
}

// What the reader keeps for the operating system: the file descriptor of the shared-memory object on Unix targets, the mapping
// handle on Windows. Only `reader_release_channel` should close it.
Read_Channel_Os_Specific :: struct {
    #if OS == .WINDOWS mapping: Channel_Native.HANDLE;
    else shm: s32;
}

// What the writer keeps for the operating system: the file descriptor of the shared-memory object on Unix targets, the mapping
// handle on Windows. Only `writer_disconnect` should close it.
Write_Channel_Os_Specific :: struct {
    #if OS == .WINDOWS mapping: Channel_Native.HANDLE;
    else shm: s32;
}

// The reading end of a channel, from `reader_create_channel`. It points at the shared region through `header`. Give it back with
// `reader_release_channel`; the pointer is not valid after that. Keep the record where it is while in use, and do not copy it for
// a second reader: a channel has exactly one reader.
Read_Channel :: struct {
    header: *Header;
    os_specific: Read_Channel_Os_Specific;
}

// The writing end of a channel, from `writer_connect`. It points at the shared region through `header`. Give it back with
// `writer_disconnect`. A channel has exactly one writer at a time.
Write_Channel :: struct {
    header: *Header;
    os_specific: Write_Channel_Os_Specific;
}

// The two cursors of the message ring, in bytes counted from the start of the channel's life: `read_cursor` is how much the
// reader has finished with and `write_cursor` how much the writer has committed. They only grow; the position in the ring is the
// cursor modulo `data_size_in_bytes`, and `write_cursor - read_cursor` is what is waiting. They sit in different cache lines.
// Get the record with `get_message_queue_header`.
Message_Queue_Header :: struct {
    read_cursor: s64;
    pad: [56] u8 = ---;
    write_cursor: s64;
}

// Space in the ring that the writer has claimed with `writer_reserve_message` and not committed yet.
//
// Write the message through `user_write_pointer`, as many bytes as were asked for, then pass the record to
// `writer_commit_message`. The rest of the fields are the channel's bookkeeping and must be left as they are. Only one
// reservation can be open at a time: reserving again before committing gives the same space.
Reserved_Message :: struct {
    channel: *Write_Channel;
    offset: s64;
    total_bytes: s64;
    user_write_pointer: *u8;
}

// A message the reader has found with `reader_poll_for_message`: `user_read_pointer` and `user_read_bytes` give its payload,
// which is read in place in the shared region. Pass it to `reader_done_with_message` when finished, which frees the space for
// the writer; the payload must not be used afterwards. Messages arrive in the order they were committed.
Read_Message :: struct {
    channel: *Read_Channel;
    offset: s64;
    total_bytes: s64;
    user_read_pointer: *u8;
    user_read_bytes: s64;
}

#scope_file
#if OS == .WINDOWS Channel_Native :: #import "internal/bindings/windows-api";
