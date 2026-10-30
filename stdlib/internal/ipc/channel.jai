// Creates the channel called `name` with a region of `bytes` bytes and returns it as `(success, channel)`. The caller becomes
// its reader and owns the name: nobody can connect until it exists, and the name goes away with `reader_release_channel`.
//
// - `bytes` is the size of the whole region, including a 256-byte header; the message ring gets `bytes - 256` rounded down to a
//   multiple of 8. The least allowed is 288, which leaves room for a 16-byte message. Pick a size for the biggest message plus
//   some room for the writer to run ahead: a message needs its payload plus 16 bytes, rounded up to 8, and never wraps.
// - `name` is a short non-empty text. On Unix targets it names a POSIX shared-memory object: one leading `/` is allowed and
//   added if missing, no other `/`, and the whole name with the slash is at most 31 bytes on macOS. On Windows it names a
//   mapping object. No NUL bytes.
//
// It fails if the name is taken (only one reader per name), the size is too small, or the system refuses; nothing is left behind
// then. Uses `context.allocator` for a small record that `reader_release_channel` frees.
//
// ```jai,skip_on(wasm)
// name := tprint("jai-doc-%", to_nanoseconds(current_time_monotonic()));
//
// ok, reader := reader_create_channel(name, 1024);
// assert(ok);
// assert(reader.header.data_size_in_bytes == 768);
//
// taken, again := reader_create_channel(name, 1024);
// assert(!taken); // the name belongs to the first reader
//
// assert(reader_release_channel(*reader));
// ```
reader_create_channel :: (name: string, bytes: s64) -> (success: bool, channel: Read_Channel) {
    return reader_create_channel_os_specific(name, bytes);
}

// The `Message_Queue_Header` of a channel, given its `Header`: a pointer into the shared region, valid until the channel is released.
// Handy for asking how much is waiting (`write_cursor - read_cursor`).
get_message_queue_header :: inline (header: *Header) -> *Message_Queue_Header {
    return cast(*Message_Queue_Header) (cast(*u8) header + header.offset_to_message_queue_header);
}

// Claims room for a message of `data_bytes` bytes and returns `(success, reserved)`; write the payload into
// `reserved.user_write_pointer`, then call `writer_commit_message`. Nothing is visible to the reader until the commit.
//
// It does not wait. It fails (`success` is `false`) when the ring has no room right now, in which case try again after
// the reader has caught up; when the message could never fit (more than the ring's capacity less 16 bytes); when
// `data_bytes` is negative; when the reader has disconnected; or when the channel is corrupt. At the end of the ring a message
// that does not fit moves to the start, and the gap is skipped by the reader. `data_bytes` of `0` is a valid, empty message.
// Reserving again before committing hands out the same space, so use one reservation at a time.
writer_reserve_message :: (channel: *Write_Channel, data_bytes: s64) -> (success: bool, reserved: Reserved_Message) {
    header := channel.header;
    if !valid_header(header) || data_bytes < 0 || data_bytes > header.data_size_in_bytes - PER_MESSAGE_OVERHEAD_SIZE return false, .{};
    if load_atomic(*header.reader_status) == .DISCONNECTED || load_atomic(*header.writer_status) != .ACTIVELY_WRITING return false, .{};

    queue := get_message_queue_header(header);
    read := load_atomic(*queue.read_cursor);
    write := load_atomic(*queue.write_cursor);
    capacity := header.data_size_in_bytes;

    if !valid_cursors(read, write, capacity) return false, .{};

    total := (data_bytes + PER_MESSAGE_OVERHEAD_SIZE + 7) & ~cast(s64) 7;
    tail := capacity - write % capacity;
    gap: s64;

    if tail < total gap = tail;
    if write > 0x7fffffffffffffff - total - gap return false, .{};

    available := capacity - (write - read);
    if total + gap > available {
        if gap > 0 && gap <= available {
            if gap >= PER_MESSAGE_OVERHEAD_SIZE {
                sentinel := cast(*u64) (message_data(header) + write % capacity);
                sentinel[0] = WRAPPING_SENTINEL;
                sentinel[1] = 0;
            }

            advanced := compare_and_swap(*queue.write_cursor, write, write + gap);
            if !advanced return false, .{};
        }

        return false, .{};
    }

    if gap >= PER_MESSAGE_OVERHEAD_SIZE {
        sentinel := cast(*u64) (message_data(header) + write % capacity);
        sentinel[0] = WRAPPING_SENTINEL;
        sentinel[1] = 0;
    }

    offset := write + gap;
    packet := cast(*u64) (message_data(header) + offset % capacity);
    packet[0] = MESSAGE_WORD;
    packet[1] = cast(u64) data_bytes;

    return true, .{ channel, offset, total, cast(*u8) packet + PER_MESSAGE_OVERHEAD_SIZE };
}

// Publishes a reserved message: from this call on the reader can find it. Returns `false` if the reservation is not valid
// any more (it was not made by this writer on this channel, the record was changed, or the reader has disconnected), and
// the message is then not published.
//
// Write all of the payload before calling it. After a successful commit the `Reserved_Message` must not be used again.
writer_commit_message :: (channel: *Write_Channel, message: Reserved_Message) -> success: bool {
    header := channel.header;
    if message.channel != channel || !valid_header(header) return false;

    queue := get_message_queue_header(header);
    write := load_atomic(*queue.write_cursor);
    read := load_atomic(*queue.read_cursor);
    capacity := header.data_size_in_bytes;

    if !valid_cursors(read, write, capacity) || message.offset < write || message.total_bytes < PER_MESSAGE_OVERHEAD_SIZE return false;

    gap := message.offset - write;
    if gap != 0 && gap != capacity - write % capacity return false;
    if message.total_bytes > capacity - gap - (write - read) || write > 0x7fffffffffffffff - message.total_bytes - gap return false;

    packet := cast(*u64) (message_data(header) + message.offset % capacity);
    if message.user_write_pointer != cast(*u8) packet + PER_MESSAGE_OVERHEAD_SIZE || packet[0] != MESSAGE_WORD return false;
    if packet[1] > cast(u64) capacity - PER_MESSAGE_OVERHEAD_SIZE || message.total_bytes != (cast(s64) packet[1] + PER_MESSAGE_OVERHEAD_SIZE + 7) & ~cast(s64) 7 return false;
    if message.total_bytes > capacity - message.offset % capacity || load_atomic(*header.reader_status) == .DISCONNECTED return false;

    return compare_and_swap(*queue.write_cursor, write, message.offset + message.total_bytes);
}

// Looks for the next message without waiting and returns `(found, fatal_error, message)`.
//
// - `found` is `true` with the message when one is waiting. Read the payload at `message.user_read_pointer` for
//   `message.user_read_bytes` bytes, then call `reader_done_with_message`. Until then the same message is returned again.
// - `found` and `fatal_error` both `false` mean the ring is empty; poll again later (sleep, or use whatever wake-up the program has).
// - `fatal_error` `true` means the shared region is corrupt (the other process wrote over the header or the cursors); the
//   channel cannot be used further and should be released.
//
// ```jai,skip_on(wasm)
// name := tprint("jai-doc-%", to_nanoseconds(current_time_monotonic()));
//
// ok, reader := reader_create_channel(name, 1024);
// assert(ok);
// connected, writer := writer_connect(name);
// assert(connected);
//
// // The writer reserves space, fills it in and commits it.
// reserved_ok, reserved := writer_reserve_message(*writer, 5);
// assert(reserved_ok);
// memcpy(reserved.user_write_pointer, "hello".data, 5);
// assert(writer_commit_message(*writer, reserved));
//
// // The reader finds it, reads it in place and gives the space back.
// found, fatal, message := reader_poll_for_message(*reader);
// assert(found && !fatal);
// assert(to_string(message.user_read_pointer, message.user_read_bytes) == "hello");
// assert(reader_done_with_message(*message));
//
// found, fatal, message = reader_poll_for_message(*reader);
// assert(!found && !fatal); // empty again
//
// assert(writer_disconnect(*writer));
// assert(reader_release_channel(*reader));
// ```
reader_poll_for_message :: (channel: *Read_Channel) -> (found: bool, fatal_error: bool, message: Read_Message) {
    header := channel.header;
    if !valid_header(header) return false, true, .{};

    queue := get_message_queue_header(header);
    capacity := header.data_size_in_bytes;
    cursor := load_atomic(*queue.read_cursor);
    write := load_atomic(*queue.write_cursor);

    if !valid_cursors(cursor, write, capacity) return false, true, .{};
    if cursor == write return false, false, .{};

    tail := capacity - cursor % capacity;
    skip := tail < PER_MESSAGE_OVERHEAD_SIZE;
    packet := cast(*u64) (message_data(header) + cursor % capacity);

    if !skip skip = packet[0] == WRAPPING_SENTINEL;

    if skip {
        if tail == capacity || write - cursor < tail return false, true, .{};

        advanced := compare_and_swap(*queue.read_cursor, cursor, cursor + tail);
        if !advanced return false, true, .{};

        cursor += tail;
        if cursor == write return false, false, .{};

        tail = capacity;
        packet = cast(*u64) message_data(header);
    }

    if write - cursor < PER_MESSAGE_OVERHEAD_SIZE || packet[0] != MESSAGE_WORD || packet[1] > cast(u64) capacity - PER_MESSAGE_OVERHEAD_SIZE return false, true, .{};

    payload := cast(s64) packet[1];
    total := (payload + PER_MESSAGE_OVERHEAD_SIZE + 7) & ~cast(s64) 7;

    if total > tail || total > write - cursor return false, true, .{};

    return true, false, .{ channel, cursor, total, cast(*u8) packet + PER_MESSAGE_OVERHEAD_SIZE, payload };
}

// Marks the message as read and frees its space for the writer; the payload must not be used afterwards. Messages have to be
// finished in order, so this works on the message `reader_poll_for_message` returned last. Returns `false`, and frees nothing,
// for a message that was already finished (the record is cleared on success) or whose bookkeeping does not match the channel.
reader_done_with_message :: (message: *Read_Message) -> bool {
    if !message.channel || !valid_header(message.channel.header) return false;

    header := message.channel.header;
    queue := get_message_queue_header(header);
    read := load_atomic(*queue.read_cursor);
    write := load_atomic(*queue.write_cursor);
    capacity := header.data_size_in_bytes;

    if !valid_cursors(read, write, capacity) || read != message.offset || message.total_bytes < PER_MESSAGE_OVERHEAD_SIZE || message.total_bytes > write - read return false;
    if message.user_read_bytes < 0 || message.user_read_bytes > message.total_bytes - PER_MESSAGE_OVERHEAD_SIZE ||
       message.user_read_pointer != message_data(header) + read % capacity + PER_MESSAGE_OVERHEAD_SIZE return false;

    packet := cast(*u64) (message_data(header) + read % capacity);
    if message.total_bytes > capacity - read % capacity || packet[0] != MESSAGE_WORD || packet[1] != cast(u64) message.user_read_bytes return false;
    if message.total_bytes != ((message.user_read_bytes + PER_MESSAGE_OVERHEAD_SIZE + 7) & ~cast(s64) 7) return false;
    if !compare_and_swap(*queue.read_cursor, read, read + message.total_bytes) return false;

    message.* = .{};

    return true;
}

#scope_module
#import "Basic";
#if OS == .WINDOWS Channel_Windows :: #import "internal/bindings/windows-api";

CHANNEL_WORD :: 0x436d6853;
MESSAGE_WORD :: 0x323230324d6d6853;

Registration :: struct {
    next: *Registration;
    header: *Header;
    #if OS == .WINDOWS mapping: Channel_Windows.HANDLE;
    else fd: s32;
    extent: s64;
    reader: bool;
    allocator: Allocator;
    name: string;
}

registrations: *Registration;
registry_lock: s32;

remove_registration :: (header: *Header, reader: bool) -> *Registration {
    lock_registry();
    defer unlock_registry();
    previous: *Registration;
    record := registrations;

    while record && (record.header != header || record.reader != reader) {
        previous = record;
        record = record.next;
    }

    if !record return null;
    if previous previous.next = record.next;
    else registrations = record.next;

    return record;
}

allocate_registration :: (name: string) -> *Registration {
    if name.count <= 0 || name.count > 0x7fffffffffffffff - size_of(Registration) - 2 return null;

    for byte, index: name {
        if byte == 0 return null;
        #if OS != .WINDOWS if byte == #char "/" && index != 0 return null;
    }

    extra: s64;

    #if OS != .WINDOWS if name[0] != #char "/" extra = 1;

    count := name.count + extra;
    allocator := context.allocator;
    record := cast(*Registration) allocator.proc(.ALLOCATE, size_of(Registration) + count + 1, 0, null, allocator.data);

    if !record return null;

    record.* = .{};
    record.allocator = allocator;
    record.name = .{ count, cast(*u8) record + size_of(Registration) };
    if extra != 0 record.name.data[0] = #char "/";

    memcpy(record.name.data + extra, name.data, name.count);
    record.name.data[count] = 0;

    return record;
}

release_registration :: (record: *Registration) {
    allocator := record.allocator;
    allocator.proc(.FREE, 0, size_of(Registration) + record.name.count + 1, record, allocator.data);
} @FreesArgument

lock_registry :: () {
    acquired := compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1);
    while !acquired acquired = compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1);
}

unlock_registry :: () {
    released := compare_and_swap(*registry_lock, cast(s32) 1, cast(s32) 0);
    assert(released);
}

load_atomic :: (pointer: *$T) -> T {
    zero: T;
    _, value := compare_and_swap(pointer, zero, zero);
    return value;
}

store_atomic :: (pointer: *$T, value: T) {
    old := load_atomic(pointer);
    success := false;

    while !success {
        observed: T;
        success, observed = compare_and_swap(pointer, old, value);
        if !success old = observed;
    }
}

known_extent :: (header: *Header) -> s64 {
    lock_registry();
    defer unlock_registry();
    record := registrations;
    while record {
        if record.header == header return record.extent;
        record = record.next;
    }

    return 0;
}

valid_header :: (header: *Header, extent: s64 = -1) -> bool {
    if extent < 0 extent = known_extent(header);
    if extent < OFFSET_TO_MESSAGE_DATA + 32 return false;
    if !header || load_atomic(*header.magic) != CHANNEL_WORD || header.version != PROTOCOL_VERSION return false;
    if header.offset_to_os_specific_header != OFFSET_TO_HEADER_OS_SPECIFIC || header.offset_to_message_queue_header != OFFSET_TO_MESSAGE_QUEUE_HEADER || header.offset_to_message_data != OFFSET_TO_MESSAGE_DATA return false;
    if (cast(*Header_Os_Specific) (cast(*u8) header + OFFSET_TO_HEADER_OS_SPECIFIC)).magic != CHANNEL_WORD return false;

    reader := cast(s32) load_atomic(*header.reader_status);
    writer := cast(s32) load_atomic(*header.writer_status);

    if reader < 0 || reader > 2 || writer < 0 || writer > 2 return false;

    return header.full_memory_region_size_in_bytes == extent && header.data_size_in_bytes >= 32 &&
           header.data_size_in_bytes <= header.full_memory_region_size_in_bytes - OFFSET_TO_MESSAGE_DATA && header.data_size_in_bytes % 8 == 0;
}

valid_cursors :: (read: s64, write: s64, capacity: s64) -> bool {
    return read >= 0 && write >= read && write - read <= capacity && read % 8 == 0 && write % 8 == 0;
}

message_data :: (header: *Header) -> *u8 {
    return cast(*u8) header + header.offset_to_message_data;
}

log_cleanup :: () {
    context.logger("shared-memory cleanup failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
}
