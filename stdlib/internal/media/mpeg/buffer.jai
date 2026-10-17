// Reviewed public shapes; field/enum order is ABI-significant.

Buffer_Mode :: enum {
    FILE;
    FIXED_MEM;
    RING;
    APPEND;
}

Start_Code :: enum u8 {
    SEQUENCE :: 0xB3;
    SLICE_FIRST :: 0x01;
    SLICE_LAST :: 0xAF;
    PICTURE :: 0x00;
    EXTENSION :: 0xB5;
    USER_DATA :: 0xB2;

    PACK :: 0xBA;
    END :: 0xB9;
    SYSTEM :: 0xBB;

    PRIVATE :: 0xBD;
    AUDIO_1 :: 0xC0;
    AUDIO_2 :: 0xC1;
    AUDIO_3 :: 0xC2;
    AUDIO_4 :: 0xC3;
    VIDEO_1 :: 0xE0;

    NOT_FOUND :: 0xFF;
}

Vlc :: struct {
    index: s16;
    value: s16;
}

Vlc_Uint :: struct {
    index: s16;
    value: u16;
}

Buffer :: struct {
    bit_index: s64;
    capacity: s64;
    length: s64;
    total_size: s64;

    discard_read_bytes := false;

    has_ended := false;
    free_when_done := false;
    close_when_done := false;
    file_initted := false;

    fh: File;

    load_callback: Buffer_Load_Callback;
    load_callback_user_data: *void;

    bytes: *u8;

    mode: Buffer_Mode;
}

Buffer_Load_Callback :: #type (buffer: *Buffer, user: *void);

BUFFER_DEFAULT_SIZE :: 128 * 1024;

// Adapted from MIT-licensed PL_MPEG by Dominic Szablewski.
// Original C input/attribution: third_party/pl_mpeg. This is a separate Jai
// implementation using layouts and checked bounds/I/O; no Jai port input.

buffer_create_with_memory :: (memory: [] u8, free_when_done: bool) -> *Buffer {
    if memory.count < 0 || memory.count > MAX_BUFFER_BYTES || (memory.count > 0 && !memory.data) return null;

    buffer := B.New(Buffer);
    buffer.capacity = memory.count;
    buffer.length = memory.count;
    buffer.total_size = memory.count;
    buffer.bytes = memory.data;
    buffer.free_when_done = free_when_done;
    buffer.mode = .FIXED_MEM;

    return buffer;
}

buffer_create_with_capacity :: (capacity: s64) -> *Buffer {
    if capacity <= 0 || capacity > MAX_BUFFER_BYTES return null;

    buffer := B.New(Buffer);
    buffer.bytes = cast(*u8) B.alloc(capacity);
    if !buffer.bytes {
        B.free(buffer);
        return null;
    }

    buffer.capacity = capacity;
    buffer.free_when_done = true;
    buffer.mode = .RING;
    buffer.discard_read_bytes = true;

    return buffer;
}

create_for_appending :: (initial_capacity: s64) -> *Buffer {
    buffer := buffer_create_with_capacity(initial_capacity);
    if buffer {
        buffer.mode = .APPEND;
        buffer.discard_read_bytes = false;
    }

    return buffer;
}

buffer_create_with_filename :: (filename: string) -> *Buffer {
    file, opened := F.file_open(filename, log_errors = false);
    if !opened return null;

    buffer := buffer_create_with_file(file, true);
    if !buffer F.file_close(*file);

    return buffer;
}

buffer_create_with_file :: (fh: File, close_when_done: bool) -> *Buffer {
    if !F.is_valid(fh) return null;

    length, sized := F.file_length(fh);
    if !sized || length < 0 || length > MAX_BUFFER_BYTES || !F.file_set_position(fh, 0) return null;

    buffer := buffer_create_with_capacity(BUFFER_DEFAULT_SIZE);
    if !buffer return null;

    buffer.total_size = length;
    buffer.fh = fh;
    buffer.file_initted = true;
    buffer.close_when_done = close_when_done;
    buffer.mode = .FILE;
    buffer.load_callback = buffer_load_file_callback;

    return buffer;
}

buffer_destroy :: (using buffer: *Buffer) {
    if !buffer return;
    if file_initted && close_when_done F.file_close(*fh);
    if free_when_done B.free(bytes);
    B.free(buffer);
} @Releases

buffer_get_size :: (buffer: *Buffer) -> s64 {
    check_state(buffer);
    if buffer.mode == .FILE return buffer.total_size;
    return buffer.length;
}

buffer_get_remaining :: (using buffer: *Buffer) -> s64 {
    check_state(buffer);
    return length - (bit_index >> 3);
}

buffer_write :: (using buffer: *Buffer, memory: [] u8) -> s64 {
    check_state(buffer);
    B.assert(memory.count >= 0 && (memory.count == 0 || memory.data != null));
    if mode == .FIXED_MEM return 0;

    if discard_read_bytes {
        buffer_discard_read_bytes(buffer);
        if mode == .RING total_size = 0;
    }

    B.assert(memory.count <= MAX_BUFFER_BYTES - length);
    needed := length + memory.count;
    if needed > capacity {
        next := capacity;
        while next < needed {
            if next > MAX_BUFFER_BYTES / 2 next = needed;
            else next *= 2;
        }

        expanded := cast(*u8) B.realloc(bytes, next, capacity);
        if !expanded return 0;

        bytes = expanded;
        capacity = next;
    }

    if memory.count > 0 memcpy(bytes + length, memory.data, memory.count);

    length += memory.count;
    if mode == .RING || mode == .APPEND total_size = 0;

    has_ended = false;

    return memory.count;
}

buffer_set_load_callback :: (buffer: *Buffer, callback: Buffer_Load_Callback, user: *void) {
    check_state(buffer);
    buffer.load_callback = callback;
    buffer.load_callback_user_data = user;
}

buffer_signal_end :: (buffer: *Buffer) {
    check_state(buffer);
    buffer.total_size = buffer.length;
    if buffer.length == 0 buffer.has_ended = true;
}

buffer_seek :: (buffer: *Buffer, pos: s64) {
    check_state(buffer);
    B.assert(pos >= 0);

    if buffer.mode == .FILE {
        B.assert(buffer.file_initted && pos <= buffer.total_size);

        if !F.file_set_position(buffer.fh, pos) {
            context.logger("MPEG buffer seek failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            return;
        }

        buffer.bit_index = 0;
        buffer.length = 0;
    } else if buffer.mode == .RING {
        B.assert(pos == 0, "ring MPEG buffers can only rewind");
        buffer.bit_index = 0;
        buffer.length = 0;
        buffer.total_size = 0;
    } else {
        B.assert(pos <= buffer.length);
        buffer.bit_index = pos * 8;
    }

    buffer.has_ended = false;
}

buffer_rewind :: (buffer: *Buffer) {
    buffer_seek(buffer, 0);
}

buffer_tell :: (buffer: *Buffer) -> s64 {
    check_state(buffer);

    if buffer.mode == .FILE {
        position, valid := F.file_current_position(buffer.fh);
        B.assert(valid, "MPEG file position unavailable");
        return position + (buffer.bit_index >> 3) - buffer.length;
    }

    return buffer.bit_index >> 3;
}

buffer_discard_read_bytes :: (using buffer: *Buffer) {
    check_state(buffer);
    consumed := bit_index >> 3;
    if consumed > 0 {
        if (mode == .RING || mode == .APPEND) && total_size > 0 {
            B.assert(total_size >= consumed);
            total_size -= consumed;
            if total_size == 0 has_ended = true;
        }

        remaining := length - consumed;
        if remaining > 0 memcpy(bytes, bytes + consumed, remaining);

        bit_index -= consumed * 8;
        length = remaining;
    }
}

buffer_load_file_callback :: (using buffer: *Buffer, user: *void) {
    check_state(buffer);
    B.assert(file_initted);
    if discard_read_bytes buffer_discard_read_bytes(buffer);

    available := capacity - length;
    if available == 0 return;

    success, count := F.file_read(fh, bytes + length, available);

    length += count;

    if !success {
        context.logger("MPEG file read failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        has_ended = true;
    } else if count == 0 has_ended = true;
}

buffer_has :: (buffer: *Buffer, count: s64) -> bool {
    check_state(buffer);
    B.assert(count >= 0);
    if buffer.length * 8 - buffer.bit_index >= count return true;

    if buffer.load_callback {
        buffer.load_callback(buffer, buffer.load_callback_user_data);
        check_state(buffer);
        if buffer.length * 8 - buffer.bit_index >= count return true;
    }

    if buffer.mode == .FIXED_MEM || (buffer.total_size != 0 && buffer.length == buffer.total_size) buffer.has_ended = true;

    return false;
}

buffer_read :: (buffer: *Buffer, count: s64) -> s32 {
    B.assert(count >= 0 && count <= 32);
    if !buffer_has(buffer, count) return 0;

    value: u32;
    remaining := count;

    while remaining > 0 {
        current := cast(u32) buffer.bytes[buffer.bit_index >> 3];
        available := 8 - (buffer.bit_index & 7);
        take := B.min(available, remaining);
        shift := available - take;
        mask := cast(u32) 255 >> (8 - take);

        value = (value << take) | ((current >> shift) & mask);
        buffer.bit_index += take;
        remaining -= take;
    }

    return cast,no_check(s32) value;
}

buffer_align :: (buffer: *Buffer) {
    check_state(buffer);
    buffer.bit_index = ((buffer.bit_index + 7) >> 3) * 8;
}

buffer_skip :: (buffer: *Buffer, count: s64) {
    if buffer_has(buffer, count) buffer.bit_index += count;
}

buffer_skip_bytes :: (buffer: *Buffer, v: u8) -> s32 {
    buffer_align(buffer);
    skipped: s32;

    while buffer_has(buffer, 8) && buffer.bytes[buffer.bit_index >> 3] == v {
        buffer.bit_index += 8;
        skipped += 1;
    }

    return skipped;
}

buffer_next_start_code :: (buffer: *Buffer) -> Start_Code {
    buffer_align(buffer);
    found := false;
    result := Start_Code.NOT_FOUND;

    while !found && buffer_has(buffer, 40) {
        index := buffer.bit_index >> 3;
        if buffer.bytes[index] == 0 && buffer.bytes[index + 1] == 0 && buffer.bytes[index + 2] == 1 {
            result = cast(Start_Code) buffer.bytes[index + 3];
            buffer.bit_index = (index + 4) * 8;
            found = true;
        } else buffer.bit_index += 8;
    }

    return result;
}

buffer_find_start_code :: (buffer: *Buffer, code: Start_Code) -> bool {
    found := false;
    finished := false;

    while !finished {
        current := buffer_next_start_code(buffer);
        found = current == code;
        finished = found || current == .NOT_FOUND;
    }

    return found;
}

buffer_no_start_code :: (buffer: *Buffer) -> bool {
    if !buffer_has(buffer, 40) return false;
    index := (buffer.bit_index + 7) >> 3;
    return buffer.bytes[index] != 0 || buffer.bytes[index + 1] != 0 || buffer.bytes[index + 2] != 1;
}

buffer_has_start_code :: (buffer: *Buffer, code: Start_Code) -> bool {
    check_state(buffer);
    saved := buffer.bit_index;
    discard := buffer.discard_read_bytes;

    buffer.discard_read_bytes = false;

    defer {
        buffer.bit_index = saved;
        buffer.discard_read_bytes = discard;
    }

    return buffer_find_start_code(buffer, code);
}

buffer_read_vlc :: (buffer: *Buffer, table: [] Vlc) -> s16 {
    index: s64;
    value: s16;
    branches: s64;
    finished := false;

    while !finished {
        B.assert(buffer_has(buffer, 1), "truncated MPEG VLC code");
        bit := cast(s64) buffer_read(buffer, 1);

        B.assert(index + bit >= 0 && index + bit < table.count, "invalid MPEG VLC table index");
        entry := table[index + bit];

        B.assert(entry.index >= 0, "invalid MPEG VLC code");
        index = entry.index;
        value = entry.value;
        branches += 1;
        B.assert(branches <= table.count, "cyclic MPEG VLC table");
        finished = index == 0;
    }

    return value;
}

buffer_read_vlc_uint :: (buffer: *Buffer, table: [] Vlc_Uint) -> u16 {
    #assert size_of(Vlc_Uint) == size_of(Vlc);
    view: [] Vlc = .{ table.count, cast(*Vlc) table.data };
    return cast,no_check(u16) buffer_read_vlc(buffer, view);
}

#scope_file
B :: #import "Basic";
#import "File";
F :: #import "File";

MAX_BUFFER_BYTES :: 0x7fffffffffffffff / 8;

check_state :: (buffer: *Buffer) {
    B.assert(buffer != null && buffer.capacity >= 0 && buffer.capacity <= MAX_BUFFER_BYTES);
    B.assert(buffer.length >= 0 && buffer.length <= buffer.capacity);
    B.assert(buffer.bit_index >= 0 && buffer.bit_index <= buffer.length * 8);
    B.assert(buffer.length == 0 || buffer.bytes != null);
}
