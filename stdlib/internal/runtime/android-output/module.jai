// Private Runtime-owned pending bytes, shared canonically with Android logging.
// The sink preserves Runtime's ordinary raw descriptor output; discard performs
// no sink call. No dependency on Basic or the Android facade is introduced.
append :: (s: string, to_standard_error := false) #no_context {
    Lock.acquire();
    defer unlock();
    append_unsynchronized(s, to_standard_error);
}

append_unsynchronized :: (s: string, to_standard_error := false) #no_context {
    if s.count < 0 || (s.count > 0 && !s.data) {
        abort();
        return;
    }

    queue := *standard_output;
    fd: s32 = 1;

    if to_standard_error {
        queue = *standard_error;
        fd = 2;
    }

    for byte: s {
        queue.bytes[queue.count] = byte;
        queue.count += 1;
        if byte == 10 || queue.count == queue.bytes.count emit(queue, fd, queue.count);
    }
}

flush :: (force := false) #no_context {
    Lock.acquire();
    defer unlock();
    flush_queue(*standard_output, 1, force);
    flush_queue(*standard_error, 2, force);
}

discard_pending_output :: () #no_context {
    Lock.acquire();
    defer unlock();
    standard_output.count = 0;
    standard_error.count = 0;
}

#scope_file
Lock :: #import "internal/runtime/output-lock";

Queue :: struct {
    bytes: [4096] u8;
    count: s64;
}

standard_output: Queue;
standard_error: Queue;
libc :: #system_library "libc";
write :: (fd: s32, data: *u8, count: u64) -> s64 #foreign libc "write";
abort :: () #foreign libc "abort";

unlock :: () #no_context {
    released := Lock.release();
    if !released abort();
}

flush_queue :: (queue: *Queue, fd: s32, force: bool) #no_context {
    amount: s64;
    if force amount = queue.count;
    else for byte, index: string.{ queue.count, queue.bytes.data } if byte == 10 amount = index + 1;
    emit(queue, fd, amount);
}

emit :: (queue: *Queue, fd: s32, amount: s64) #no_context {
    cursor: s64;
    while cursor < amount {
        written := write(fd, queue.bytes.data + cursor, cast(u64) (amount - cursor));
        if written <= 0 {
            abort();
            return;
        }

        cursor += written;
    }

    remaining := queue.count - amount;
    for index: 0..remaining - 1 queue.bytes[index] = queue.bytes[amount + index];

    queue.count = remaining;
}
