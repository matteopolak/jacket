reader_create_channel_os_specific :: (name: string, bytes: s64) -> (success: bool, channel: Read_Channel) {
    reap_pending_cleanup();
    #assert size_of(Header) <= OFFSET_TO_HEADER_OS_SPECIFIC;
    #assert OFFSET_TO_MESSAGE_QUEUE_HEADER + size_of(Message_Queue_Header) <= OFFSET_TO_MESSAGE_DATA;
    if bytes < OFFSET_TO_MESSAGE_DATA + 32 return false, .{};

    registration := allocate_registration(name);
    if !registration return false, .{};

    complete := false;
    mapping: W.HANDLE;
    view: *void;

    defer if !complete {
        registration.header = cast(*Header) view;
        registration.mapping = mapping;
        cleanup_failed_construction(registration);
    }

    wide, converted, _length := Utf8.utf8_to_wide(name);
    if !converted return false, .{};

    defer free(wide);
    extent_bits := cast(u64) bytes;

    clear_last_error(0);
    mapping = W.CreateFileMappingW(W.INVALID_HANDLE_VALUE, null, W.PAGE_READWRITE,
                                  cast,truncate(u32) (extent_bits >> 32), cast,truncate(u32) extent_bits, wide);
    if !mapping || cast(u32) W.GetLastError() == W.ERROR_ALREADY_EXISTS return false, .{};

    view = W.MapViewOfFile(mapping, W.FILE_MAP_ALL_ACCESS, 0, 0, 0);
    if !view return false, .{};

    extent := mapped_extent(view);
    if extent < bytes return false, .{};

    memset(view, 0, bytes);
    header := cast(*Header) view;
    header.version = PROTOCOL_VERSION;
    header.reader_status = .ACTIVELY_READING;
    header.offset_to_os_specific_header = OFFSET_TO_HEADER_OS_SPECIFIC;
    header.offset_to_message_queue_header = OFFSET_TO_MESSAGE_QUEUE_HEADER;
    header.offset_to_message_data = OFFSET_TO_MESSAGE_DATA;
    header.full_memory_region_size_in_bytes = extent;
    header.data_size_in_bytes = (bytes - OFFSET_TO_MESSAGE_DATA) & ~cast(s64) 7;
    (cast(*Header_Os_Specific) (cast(*u8) view + OFFSET_TO_HEADER_OS_SPECIFIC)).magic = CHANNEL_WORD;
    registration.header = header;
    registration.mapping = mapping;
    registration.extent = extent;
    registration.reader = true;
    publish_registration(registration);
    store_atomic(*header.magic, cast(u32) CHANNEL_WORD);
    complete = true;

    return true, .{ header, .{ mapping } };
}

writer_connect :: (name: string, name_not_found_is_quiet := false) -> (success: bool, channel: Write_Channel) {
    reap_pending_cleanup();
    registration := allocate_registration(name);
    if !registration return false, .{};

    complete := false;
    mapping: W.HANDLE;
    view: *void;

    defer if !complete {
        registration.header = cast(*Header) view;
        registration.mapping = mapping;
        cleanup_failed_construction(registration);
    }

    wide, converted, _length := Utf8.utf8_to_wide(name);
    if !converted return false, .{};

    defer free(wide);
    mapping = W.OpenFileMappingW(W.FILE_MAP_ALL_ACCESS, 0, wide);
    if !mapping {
        if !name_not_found_is_quiet || cast(u32) W.GetLastError() != W.ERROR_FILE_NOT_FOUND {
            context.logger("shared-memory writer open failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        }

        return false, .{};
    }

    view = W.MapViewOfFile(mapping, W.FILE_MAP_ALL_ACCESS, 0, 0, 0);
    if !view return false, .{};

    extent := mapped_extent(view);
    header := cast(*Header) view;

    if !valid_header(header, extent) || load_atomic(*header.reader_status) == .DISCONNECTED return false, .{};
    if !compare_and_swap(*header.writer_status, .DISCONNECTED, .ACTIVELY_WRITING) return false, .{};

    registration.header = header;
    registration.mapping = mapping;
    registration.extent = extent;
    publish_registration(registration);
    complete = true;

    return true, .{ header, .{ mapping } };
}

writer_disconnect :: (channel: *Write_Channel) -> bool {
    return release_windows_channel(*channel.header, *channel.os_specific.mapping, false);
}

reader_release_channel :: (channel: *Read_Channel) -> bool {
    return release_windows_channel(*channel.header, *channel.os_specific.mapping, true);
}

#scope_file
#import "Basic";
W :: #import "Windows";
Utf8 :: #import "Windows_Utf8";

kernel32 :: #system_library "kernel32";
clear_last_error :: (code: u32) #foreign kernel32 "SetLastError";

pending_cleanup: *Registration;

cleanup_failed_construction :: (record: *Registration) {
    if record.header && W.UnmapViewOfFile(record.header) != 0 record.header = null;
    if record.mapping && W.CloseHandle(record.mapping) != 0 record.mapping = null;

    if !record.header && !record.mapping {
        release_registration(record);
        return;
    }

    lock_registry();
    record.next = pending_cleanup;
    pending_cleanup = record;
    unlock_registry();
    log_cleanup();
}

reap_pending_cleanup :: () {
    lock_registry();
    record := pending_cleanup;

    pending_cleanup = null;
    unlock_registry();

    while record {
        next := record.next;
        record.next = null;
        cleanup_failed_construction(record);
        record = next;
    }
}

mapped_extent :: (view: *void) -> s64 {
    info: W.MEMORY_BASIC_INFORMATION;
    if W.VirtualQuery(view, *info, size_of(W.MEMORY_BASIC_INFORMATION)) != size_of(W.MEMORY_BASIC_INFORMATION) return -1;
    if info.BaseAddress != view || info.RegionSize > 0x7fffffffffffffff return -1;
    return cast(s64) info.RegionSize;
}

publish_registration :: (record: *Registration) {
    lock_registry();
    record.next = registrations;
    registrations = record;
    unlock_registry();
}

release_windows_channel :: (header: **Header, mapping: *W.HANDLE, reader: bool) -> bool {
    reap_pending_cleanup();
    if !header.* && !mapping.* return false;

    lock_registry();
    previous: *Registration;
    record := registrations;

    while record && (record.header != header.* || record.mapping != mapping.* || record.reader != reader) {
        previous = record;
        record = record.next;
    }

    if !record {
        unlock_registry();
        return false;
    }

    if previous previous.next = record.next;
    else registrations = record.next;

    unlock_registry();

    if header.* {
        if reader store_atomic(*header.*.reader_status, type_of(header.*.reader_status).DISCONNECTED);
        else store_atomic(*header.*.writer_status, type_of(header.*.writer_status).DISCONNECTED);
        if W.UnmapViewOfFile(header.*) != 0 header.* = null;
    }

    if mapping.* && W.CloseHandle(mapping.*) != 0 mapping.* = null;

    record.header = header.*;
    record.mapping = mapping.*;

    if header.* || mapping.* {
        publish_registration(record);
        log_cleanup();
        return false;
    }

    release_registration(record);

    return true;
}
