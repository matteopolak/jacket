# Shared_Memory_Channel


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Header :: struct`
- `Read_Channel :: struct`
- `Write_Channel :: struct`
- `Message_Queue_Header :: struct`
- `Reserved_Message :: struct`
- `Read_Message :: struct`
- `OFFSET_TO_HEADER_OS_SPECIFIC :: 64`
- `OFFSET_TO_MESSAGE_QUEUE_HEADER :: 128`
- `OFFSET_TO_MESSAGE_DATA :: 256`
- `PER_MESSAGE_OVERHEAD_SIZE :: 16`
- `WRAPPING_SENTINEL :: 0xcafebabef00dface`
- `reader_create_channel :: (name: string, bytes: s64) -> (success: bool, channel: Read_Channel)`
- `get_message_queue_header :: inline (header: *Header) -> *Message_Queue_Header`
- `writer_reserve_message :: (channel: *Write_Channel, data_bytes: s64) -> (success: bool, reserved: Reserved_Message)`
- `writer_commit_message :: (channel: *Write_Channel, message: Reserved_Message) -> success: bool`
- `reader_poll_for_message :: (channel: *Read_Channel) -> (found: bool, fatal_error: bool, message: Read_Message)`
- `reader_done_with_message :: (message: *Read_Message) -> bool`
- `PROTOCOL_VERSION :: 1`
- `CHANNEL_MAGIC :: "ShmC"`
- `MESSAGE_MAGIC :: "ShmM2022"`
- [OS == .WINDOWS] `Header_Os_Specific :: struct`
- [OS == .WINDOWS] `Read_Channel_Os_Specific :: struct`
- [OS == .WINDOWS] `Write_Channel_Os_Specific :: struct`
- [OS == .WINDOWS] `reader_create_channel_os_specific :: (name: string, bytes: s64) -> (success: bool, channel: Read_Channel)`
- [OS == .WINDOWS] `reader_release_channel :: (channel: *Read_Channel) -> bool`
- [OS == .WINDOWS] `writer_connect :: (name: string, name_not_found_is_quiet := false) -> (success: bool, channel: Write_Channel)`
- [OS == .WINDOWS] `writer_disconnect :: (channel: *Write_Channel) -> bool`
- `Header_Os_Specific :: struct`
- `Read_Channel_Os_Specific :: struct`
- `Write_Channel_Os_Specific :: struct`
- `reader_create_channel_os_specific :: (name: string, bytes: s64) -> (success: bool, channel: Read_Channel)`
- `reader_release_channel :: (channel: *Read_Channel) -> bool`
- `writer_connect :: (name: string, name_not_found_is_quiet := false) -> (success: bool, channel: Write_Channel)`
- `writer_disconnect :: (channel: *Write_Channel) -> bool`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Header`

```jai
Header :: struct {
    magic   : u32;
    version : u32;

    reader_status: enum s32 {
        DISCONNECTED     :: 0;
        PAUSED           :: 1;
        ACTIVELY_READING :: 2;
    };

    writer_status: enum s32 {
        DISCONNECTED     :: 0;
        PAUSED           :: 1;
        ACTIVELY_WRITING :: 2;
    };

    offset_to_os_specific_header:    u32;
    offset_to_message_queue_header:  u32;
    offset_to_message_data:          u32;

    full_memory_region_size_in_bytes: s64;
    data_size_in_bytes:               s64;
}
```
### `Header_Os_Specific`

```jai
Header_Os_Specific :: struct {
    magic:   u32;
}
```
### `Message_Queue_Header`

```jai
Message_Queue_Header :: struct {
    read_cursor:  s64;
    pad: [56] u8 = ---;
    write_cursor: s64;
}
```
### `Read_Channel`

```jai
Read_Channel :: struct {
    header: *Header;
    os_specific: Read_Channel_Os_Specific;
}
```
### `Read_Channel_Os_Specific`

Variant 1 (Windows):

```jai
Read_Channel_Os_Specific :: struct {
    mapping: HANDLE;
}
```
Variant 2 (common):

```jai
Read_Channel_Os_Specific :: struct {
    shm: s32;
}
```
### `Read_Message`

```jai
Read_Message :: struct {
    channel:     *Read_Channel;
    offset:      s64;
    total_bytes: s64;

    user_read_pointer: *u8;
    user_read_bytes:   s64;
}
```
### `Reserved_Message`

```jai
Reserved_Message :: struct {
    channel:     *Write_Channel;
    offset:      s64;
    total_bytes: s64;

    user_write_pointer: *u8;
}
```
### `Write_Channel`

```jai
Write_Channel :: struct {
    header: *Header;
    os_specific: Write_Channel_Os_Specific;
}
```
### `Write_Channel_Os_Specific`

Variant 1 (Windows):

```jai
Write_Channel_Os_Specific :: struct {
    mapping: HANDLE;
}
```
Variant 2 (common):

```jai
Write_Channel_Os_Specific :: struct {
    shm: s32;
}
```
## Behavioral contract

Provides a single-reader/single-writer interprocess channel backed by shared memory. Reader creates/releases the channel; writer connects, reserves a packet region, fills it, then commits it. Messages are framed in a circular buffer whose design assumes each message fits contiguously and does not wrap; only one thread may read and one may write at a time. A reserved/read packet must be completed before requesting the next one. Reader/writer status, magic, version, offsets, and capacity are stored in a shared header. Windows and POSIX implementations use different OS mapping/handle mechanisms; simultaneous duplicate instances and reconnect behavior have documented caveats. Keep the mapping/channel alive until all packet operations finish. Depends on OS shared-memory and synchronization APIs.
