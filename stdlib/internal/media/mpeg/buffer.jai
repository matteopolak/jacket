// Reviewed public shapes; field/enum order is ABI-significant.

// Where a [Buffer] gets its bytes and whether it can grow.
//
// `FILE` refills from an open file through the load callback, `FIXED_MEM` wraps caller memory that never grows,
// `RING` is a growable buffer whose consumed bytes are discarded on write, and `APPEND` is a growable buffer that
// keeps everything written.
Buffer_Mode :: enum {
    FILE;
    FIXED_MEM;
    RING;
    APPEND;
}

// The byte that follows a `00 00 01` prefix in an MPEG stream and names the next syntax element.
//
// `SEQUENCE` to `USER_DATA` are video elements, `SLICE_FIRST..SLICE_LAST` is a slice whose value minus one is its
// macroblock row, `PACK`, `END` and `SYSTEM` are program stream headers, and `PRIVATE`, `AUDIO_*` and `VIDEO_1`
// are packet stream ids. `NOT_FOUND` is a sentinel for "no start code".
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

// One node of a variable-length code table walked one bit at a time by [buffer_read_vlc].
//
// `index` is the table position of the next node pair, or 0 at a leaf, and `value` is the decoded symbol.
// A negative `index` marks an invalid code.
Vlc :: struct {
    index: s16;
    value: s16;
}

// A [Vlc] node whose decoded symbol is unsigned, for tables read with [buffer_read_vlc_uint].
Vlc_Uint :: struct {
    index: s16;
    value: u16;
}

// A byte buffer with a bit-granular read cursor that feeds the video and audio decoders.
//
// Create one with [buffer_create_with_memory], [buffer_create_with_file], [buffer_create_with_filename],
// [buffer_create_with_capacity] or [create_for_appending], and release it with [buffer_destroy].
// The fields are decoder state; use the `buffer_*` procedures instead of writing them.
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

// Wraps `memory` in a read-only `.FIXED_MEM` [Buffer] without copying it.
//
// Returns null when the slice is too large or has a null data pointer with a nonzero count. [buffer_destroy] frees
// `memory` only when `free_when_done` is true.
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

// Creates an empty growable [Buffer] in `.RING` mode that discards consumed bytes, starting at `capacity` bytes.
//
// Returns null when `capacity` is not positive, exceeds the supported maximum or cannot be allocated.
// Fill it with [buffer_write]. [buffer_destroy] frees the bytes.
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

// Creates an empty growable [Buffer] in `.APPEND` mode, which keeps every written byte so it can be rewound.
//
// Returns null when `initial_capacity` is rejected by [buffer_create_with_capacity].
create_for_appending :: (initial_capacity: s64) -> *Buffer {
    buffer := buffer_create_with_capacity(initial_capacity);
    if buffer {
        buffer.mode = .APPEND;
        buffer.discard_read_bytes = false;
    }

    return buffer;
}

// Opens the file `filename` and returns a [Buffer] that reads it in chunks, or null if it cannot be opened or sized.
//
// The buffer closes the file in [buffer_destroy].
buffer_create_with_filename :: (filename: string) -> *Buffer {
    file, opened := F.file_open(filename, log_errors = false);
    if !opened return null;

    buffer := buffer_create_with_file(file, true);
    if !buffer F.file_close(*file);

    return buffer;
}

// Returns a [Buffer] that reads the open file `fh` from its start in chunks, or null if `fh` is invalid or its
// length cannot be read.
//
// The file position is reset to 0. [buffer_destroy] closes `fh` only when `close_when_done` is true.
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

// Frees `buffer`, closing its file and freeing its bytes when it owns them. Does nothing for null.
buffer_destroy :: (using buffer: *Buffer) {
    if !buffer return;
    if file_initted && close_when_done F.file_close(*fh);
    if free_when_done B.free(bytes);
    B.free(buffer);
}

// Returns the total size of the stream in bytes for a file buffer, or the number of bytes currently held for the
// other modes.
buffer_get_size :: (buffer: *Buffer) -> s64 {
    check_state(buffer);
    if buffer.mode == .FILE return buffer.total_size;
    return buffer.length;
}

// Returns the number of loaded bytes after the read cursor, counting a partly read byte as consumed.
buffer_get_remaining :: (using buffer: *Buffer) -> s64 {
    check_state(buffer);
    return length - (bit_index >> 3);
}

// Appends `memory` to the buffer, growing it as needed, and returns the number of bytes written.
//
// Returns 0 for a `.FIXED_MEM` buffer or when growing fails. Buffers that discard read bytes drop the consumed
// bytes first. Clears the ended flag. Asserts when the size would exceed the supported maximum.
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

// Sets the procedure that [buffer_has] calls to load more data when the cursor runs out, and the `user` pointer
// passed to it.
buffer_set_load_callback :: (buffer: *Buffer, callback: Buffer_Load_Callback, user: *void) {
    check_state(buffer);
    buffer.load_callback = callback;
    buffer.load_callback_user_data = user;
}

// Marks the bytes written so far as the whole stream, so reads that run out report the end of the buffer.
buffer_signal_end :: (buffer: *Buffer) {
    check_state(buffer);
    buffer.total_size = buffer.length;
    if buffer.length == 0 buffer.has_ended = true;
}

// Moves the read cursor to byte offset `pos` and clears the ended flag.
//
// A file buffer seeks the file and drops the loaded bytes, and asserts that `pos` is within the file. A `.RING`
// buffer only accepts 0. Other buffers assert that `pos` is within the held bytes.
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

// Moves the read cursor back to the start; see [buffer_seek].
buffer_rewind :: (buffer: *Buffer) {
    buffer_seek(buffer, 0);
}

// Returns the byte offset of the read cursor, counted from the start of the stream for file buffers.
buffer_tell :: (buffer: *Buffer) -> s64 {
    check_state(buffer);

    if buffer.mode == .FILE {
        position, valid := F.file_current_position(buffer.fh);
        B.assert(valid, "MPEG file position unavailable");
        return position + (buffer.bit_index >> 3) - buffer.length;
    }

    return buffer.bit_index >> 3;
}

// Drops the bytes before the read cursor and moves the remaining bytes to the front. Does nothing when no whole
// byte has been consumed.
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

// Reads the next chunk of the file into the free space of a file buffer; used as its load callback.
//
// Sets the ended flag when the file is exhausted or a read fails, and logs an error on failure. `user` is unused.
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

// Returns whether at least `count` bits are available after the cursor, calling the load callback once if not.
//
// Sets the ended flag when the buffer is a fixed memory buffer or its whole stream is already loaded.
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

// Reads `count` bits (0 to 32) as a big-endian value and advances the cursor.
//
// Returns 0 without advancing when fewer than `count` bits are available. Asserts when `count` is outside 0 to 32.
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

// Moves the cursor up to the next byte boundary.
buffer_align :: (buffer: *Buffer) {
    check_state(buffer);
    buffer.bit_index = ((buffer.bit_index + 7) >> 3) * 8;
}

// Advances the cursor by `count` bits when that many are available, otherwise does nothing.
buffer_skip :: (buffer: *Buffer, count: s64) {
    if buffer_has(buffer, count) buffer.bit_index += count;
}

// Aligns the cursor to a byte, then skips every following byte equal to `v` and returns how many were skipped.
buffer_skip_bytes :: (buffer: *Buffer, v: u8) -> s32 {
    buffer_align(buffer);
    skipped: s32;

    while buffer_has(buffer, 8) && buffer.bytes[buffer.bit_index >> 3] == v {
        buffer.bit_index += 8;
        skipped += 1;
    }

    return skipped;
}

// Aligns the cursor, scans for the next `00 00 01` prefix, consumes it with its code byte and returns that code.
//
// Returns `.NOT_FOUND` when the data runs out first.
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

// Advances through start codes until one equals `code` and returns true, or returns false when the data runs out.
//
// On success the cursor is just after the matching start code.
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

// Returns true when at least 40 bits follow the cursor and the next aligned bytes are not a start code prefix.
// Does not move the cursor.
buffer_no_start_code :: (buffer: *Buffer) -> bool {
    if !buffer_has(buffer, 40) return false;
    index := (buffer.bit_index + 7) >> 3;
    return buffer.bytes[index] != 0 || buffer.bytes[index + 1] != 0 || buffer.bytes[index + 2] != 1;
}

// Returns whether a start code equal to `code` appears ahead of the cursor, then restores the cursor.
//
// Loaded bytes are kept while it scans, even for buffers that normally discard read bytes.
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

// Decodes one variable-length code from `table`, reading one bit at a time, and returns its value.
//
// Asserts on truncated input or a code the table marks invalid.
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

// Like [buffer_read_vlc] for a table of [Vlc_Uint] entries, returning an unsigned value.
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
