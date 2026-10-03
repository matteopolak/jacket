reader_create_channel_os_specific :: (name: string, bytes: s64) -> (success: bool, channel: Read_Channel) {
    #assert size_of(Header) <= OFFSET_TO_HEADER_OS_SPECIFIC;
    #assert OFFSET_TO_MESSAGE_QUEUE_HEADER + size_of(Message_Queue_Header) <= OFFSET_TO_MESSAGE_DATA;
    if bytes < OFFSET_TO_MESSAGE_DATA + 32 return false, .{ os_specific = .{ shm = -1 } };

    registration := allocate_registration(name);
    if !registration return false, .{ os_specific = .{ shm = -1 } };

    completed := false;
    fd: s32 = -1;
    mapping: *void;
    extent := bytes;

    defer if !completed {
        if mapping && P.munmap(mapping, cast(u64) extent) != 0 log_cleanup();

        if fd >= 0 {
            if P.close(fd) != 0 log_cleanup();
            if P.shm_unlink(registration.name.data) != 0 log_cleanup();
        }

        release_registration(registration);
    }

    fd = P.shm_open(registration.name.data, P.O_RDWR | P.O_CREAT | P.O_EXCL, cast(P.mode_t) 384);
    if fd < 0 return false, .{ os_specific = .{ shm = -1 } };
    if P.ftruncate(fd, bytes) != 0 return false, .{ os_specific = .{ shm = -1 } };

    stat: P.stat_t;
    if P.fstat(fd, *stat) != 0 || stat.st_size < bytes return false, .{ os_specific = .{ shm = -1 } };

    extent = stat.st_size;
    mapping = P.mmap(null, cast(u64) extent, P.PROT_READ | P.PROT_WRITE, P.MAP_SHARED, fd, 0);
    if mapping == P.MAP_FAILED {
        mapping = null;
        return false, .{ os_specific = .{ shm = -1 } };
    }

    memset(mapping, 0, bytes);
    header := cast(*Header) mapping;
    header.version = PROTOCOL_VERSION;
    header.reader_status = .ACTIVELY_READING;
    header.offset_to_os_specific_header = OFFSET_TO_HEADER_OS_SPECIFIC;
    header.offset_to_message_queue_header = OFFSET_TO_MESSAGE_QUEUE_HEADER;
    header.offset_to_message_data = OFFSET_TO_MESSAGE_DATA;
    header.full_memory_region_size_in_bytes = extent;
    header.data_size_in_bytes = (bytes - OFFSET_TO_MESSAGE_DATA) & ~cast(s64) 7;
    (cast(*Header_Os_Specific) (cast(*u8) header + OFFSET_TO_HEADER_OS_SPECIFIC)).magic = CHANNEL_WORD;
    registration.header = header;
    registration.fd = fd;
    registration.extent = extent;
    registration.reader = true;
    lock_registry();
    registration.next = registrations;
    registrations = registration;
    unlock_registry();
    store_atomic(*header.magic, cast(u32) CHANNEL_WORD);
    completed = true;

    return true, .{ header, .{ fd } };
}

writer_connect :: (name: string, name_not_found_is_quiet := false) -> (success: bool, channel: Write_Channel) {
    registration := allocate_registration(name);
    if !registration return false, .{ os_specific = .{ shm = -1 } };

    completed := false;

    defer if !completed release_registration(registration);
    fd := P.shm_open(registration.name.data, P.O_RDWR, cast(P.mode_t) 0);
    if fd < 0 {
        if !name_not_found_is_quiet || cast(s32) P.errno() != P.ENOENT context.logger("shared-memory writer open failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false, .{ os_specific = .{ shm = -1 } };
    }

    mapping: *void;
    extent: s64;

    defer if !completed {
        if mapping && P.munmap(mapping, cast(u64) extent) != 0 log_cleanup();
        if P.close(fd) != 0 log_cleanup();
    }

    stat: P.stat_t;
    if P.fstat(fd, *stat) != 0 return false, .{ os_specific = .{ shm = -1 } };

    extent = stat.st_size;
    if extent < OFFSET_TO_MESSAGE_DATA + 32 return false, .{ os_specific = .{ shm = -1 } };

    mapping = P.mmap(null, cast(u64) extent, P.PROT_READ | P.PROT_WRITE, P.MAP_SHARED, fd, 0);
    if mapping == P.MAP_FAILED {
        mapping = null;
        return false, .{ os_specific = .{ shm = -1 } };
    }

    header := cast(*Header) mapping;
    if !valid_header(header, extent) return false, .{ os_specific = .{ shm = -1 } };
    if load_atomic(*header.reader_status) == .DISCONNECTED return false, .{ os_specific = .{ shm = -1 } };

    connected := compare_and_swap(*header.writer_status, .DISCONNECTED, .ACTIVELY_WRITING);
    if !connected return false, .{ os_specific = .{ shm = -1 } };

    registration.header = header;
    registration.fd = fd;
    registration.extent = extent;
    lock_registry();
    registration.next = registrations;
    registrations = registration;
    unlock_registry();
    completed = true;

    return true, .{ header, .{ fd } };
}

writer_disconnect :: (channel: *Write_Channel) -> bool {
    if !channel.header return false;

    record := remove_registration(channel.header, false);
    if !record return false;

    defer release_registration(record);
    header := record.header;

    store_atomic(*header.writer_status, type_of(header.writer_status).DISCONNECTED);
    unmapped := P.munmap(header, cast(u64) record.extent) == 0;
    closed := P.close(record.fd) == 0;

    if unmapped channel.header = null;
    if closed channel.os_specific.shm = -1;

    return unmapped && closed;
}

reader_release_channel :: (channel: *Read_Channel) -> bool {
    if !channel.header return false;

    record := remove_registration(channel.header, true);
    if !record return false;

    defer release_registration(record);
    header := channel.header;

    store_atomic(*header.reader_status, type_of(header.reader_status).DISCONNECTED);
    bytes := record.extent;
    unlinked := P.shm_unlink(record.name.data) == 0;
    unmapped := P.munmap(header, cast(u64) bytes) == 0;
    closed := P.close(record.fd) == 0;

    if unmapped channel.header = null;
    if closed channel.os_specific.shm = -1;

    return unlinked && unmapped && closed;
}

#scope_file
#import "Basic";
P :: #import "POSIX";
