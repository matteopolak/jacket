PROTOCOL_VERSION :: 1;
CHANNEL_MAGIC :: "ShmC";
MESSAGE_MAGIC :: "ShmM2022";
WRAPPING_SENTINEL :: 0xcafebabef00dface;
OFFSET_TO_HEADER_OS_SPECIFIC :: 64;
OFFSET_TO_MESSAGE_QUEUE_HEADER :: 128;
OFFSET_TO_MESSAGE_DATA :: 256;
PER_MESSAGE_OVERHEAD_SIZE :: 16;

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

Header_Os_Specific :: struct {
    magic: u32;
}

Read_Channel_Os_Specific :: struct {
    #if OS == .WINDOWS mapping: Channel_Native.HANDLE;
    else shm: s32;
}

Write_Channel_Os_Specific :: struct {
    #if OS == .WINDOWS mapping: Channel_Native.HANDLE;
    else shm: s32;
}

Read_Channel :: struct {
    header: *Header;
    os_specific: Read_Channel_Os_Specific;
}

Write_Channel :: struct {
    header: *Header;
    os_specific: Write_Channel_Os_Specific;
}

Message_Queue_Header :: struct {
    read_cursor: s64;
    pad: [56] u8 = ---;
    write_cursor: s64;
}

Reserved_Message :: struct {
    channel: *Write_Channel;
    offset: s64;
    total_bytes: s64;
    user_write_pointer: *u8;
}

Read_Message :: struct {
    channel: *Read_Channel;
    offset: s64;
    total_bytes: s64;
    user_read_pointer: *u8;
    user_read_bytes: s64;
}

#scope_file
#if OS == .WINDOWS Channel_Native :: #import "Windows";
