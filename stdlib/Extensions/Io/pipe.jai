// In-memory streams: a connected pair of ends that carry bytes between two fibers of one
// scheduler, with the same shape as a TCP connection (read what is there, 0 at the end, writes
// that never lose bytes) and none of the system calls. Tests use them to run both halves of a
// protocol in one process; nothing about the order of events depends on timing.
//
//     client, server := make_pipe();
//
//     Io.spawn(echo, server);
//
//     write(client, "ping");
//     count, error := read(client, buffer);       // "ping" echoed back
//
// Writes never block: bytes wait in memory until the other end reads them. A read waits for
// bytes, honours `with_deadline` (`.TIMED_OUT`, also for a call made after the limit) and `cancel` like a socket read, and returns 0
// once the other end has closed (or shut down its writing side) and everything sent was read.
// Both ends must be used by fibers of the same scheduler.

// One direction of a pipe.
Pipe_Half :: struct {
    data: [..] u8;
    // Bytes of `data` already read.
    start: s64;
    // The writer has nothing more to send.
    finished: bool;
    // The reader is gone: writes fail.
    abandoned: bool;
    // Fibers waiting for bytes (at most one in practice).
    readers: Waiter;
}

// One end of a pipe made by `make_pipe`.
Pipe_Stream :: struct {
    inbound: *Pipe_Half;
    outbound: *Pipe_Half;
    closed: bool;
    link: *Pipe_Link;
}

#scope_module

Pipe_Link :: struct {
    halves: [2] Pipe_Half;
    ends: [2] Pipe_Stream;
    open: s64;
    allocator: Allocator;
}

#scope_export

// Two connected ends. What one writes the other reads. `close` both when done (they free
// themselves once both are closed).
make_pipe :: () -> *Pipe_Stream, *Pipe_Stream {
    link := New(Pipe_Link);

    link.allocator = context.allocator;
    link.open = 2;

    link.ends[0].link = link;
    link.ends[0].inbound = *link.halves[0];
    link.ends[0].outbound = *link.halves[1];

    link.ends[1].link = link;
    link.ends[1].inbound = *link.halves[1];
    link.ends[1].outbound = *link.halves[0];

    return *link.ends[0], *link.ends[1];
}

// Waits for bytes and reads what is there, up to `buffer.count`. 0 means the other end closed
// and everything it sent has been read.
read :: (stream: *Pipe_Stream, buffer: [] u8) -> s64, Io_Error {
    if buffer.count == 0 return 0, .NONE;
    if deadline_passed() return 0, .TIMED_OUT;

    while true {
        if stream.closed return 0, .CLOSED;

        half := stream.inbound;
        available := half.data.count - half.start;

        if available > 0 {
            count := min(available, buffer.count);

            memcpy(buffer.data, half.data.data + half.start, count);

            half.start += count;

            if half.start == half.data.count {
                half.data.count = 0;
                half.start = 0;
            } else if half.start >= 65536 && half.start * 2 >= half.data.count {
                remaining := half.data.count - half.start;

                memcpy(half.data.data, half.data.data + half.start, remaining);

                half.data.count = remaining;
                half.start = 0;
            }

            return count, .NONE;
        }

        if half.finished return 0, .NONE;

        error := pipe_wait(half);

        if error != .NONE return 0, error;
    }

    return 0, .NONE;
}

// Reads until `buffer` is full; `.END_OF_FILE` if the other end closed first.
read_full :: (stream: *Pipe_Stream, buffer: [] u8) -> s64, Io_Error {
    return read_full(reader_of(stream), buffer);
}

// Hands `data` to the other end. Never waits. `.BROKEN_PIPE` if the other end is closed.
write :: (stream: *Pipe_Stream, data: [] u8) -> s64, Io_Error {
    return write_some(stream, data);
}

// The same: a pipe never waits, so `write_some` takes all of `data` too.
write_some :: (stream: *Pipe_Stream, data: [] u8) -> s64, Io_Error {
    if stream.closed return 0, .CLOSED;
    if data.count > 0 && deadline_passed() return 0, .TIMED_OUT;

    half := stream.outbound;

    if half.finished return 0, .BROKEN_PIPE;
    if half.abandoned return 0, .BROKEN_PIPE;
    if data.count == 0 return 0, .NONE;

    half.data.allocator = stream.link.allocator;

    array_reserve(*half.data, half.data.count + data.count);

    memcpy(half.data.data + half.data.count, data.data, data.count);

    half.data.count += data.count;

    pipe_wake(half, 0);

    return data.count, .NONE;
}

write :: (stream: *Pipe_Stream, text: string) -> s64, Io_Error {
    return write(stream, .{ text.count, text.data });
}

// This end will send nothing more: the other end reads what was sent, then 0.
shutdown_write :: (stream: *Pipe_Stream) -> Io_Error {
    if stream.closed return .CLOSED;

    stream.outbound.finished = true;

    pipe_wake(stream.outbound, 0);

    return .NONE;
}

// Closes this end: the other end reads what was already sent and then 0, writes to it fail, and
// a fiber waiting in `read` on this end returns `.CLOSED`. Close each end once: the memory of both
// goes when the second one is closed.
close :: (stream: *Pipe_Stream, now := false) -> Io_Error {
    stream.closed = true;
    stream.outbound.finished = true;
    stream.inbound.abandoned = true;

    pipe_wake(stream.outbound, 0);
    pipe_wake(stream.inbound, 1);

    link := stream.link;

    link.open -= 1;

    if link.open == 0 {
        array_reset(*link.halves[0].data);
        array_reset(*link.halves[1].data);

        release_owned(link, link.allocator);
    }

    return .NONE;
} @Releases

#scope_module

// Whether the running fiber's `with_deadline` limit has passed: a pipe call then fails before it
// starts, like a socket call does.
deadline_passed :: () -> bool {
    if !in_scheduler() return false;

    fiber := current_fiber();

    return fiber.deadline != 0 && now_nanoseconds() >= fiber.deadline;
}

// Wakes the readers of `half`; `reason` 1 tells them their own end was closed.
pipe_wake :: (half: *Pipe_Half, reason: s32) {
    while true {
        waiter := waiter_pop(*half.readers);
        if !waiter break;

        (cast(*s32) waiter.slot).* = reason;
        waiter.completed = true;

        wake(waiter.fiber);
    }
}

// Parks the running fiber until bytes arrive or the half changes, a deadline passes or it is
// canceled. `.NONE` means "look again".
pipe_wait :: (half: *Pipe_Half) -> Io_Error {
    fiber, scheduler := current_fiber();

    if fiber.canceled return .CANCELED;

    if fiber.deadline != 0 {
        if fiber.deadline <= now_nanoseconds() return .TIMED_OUT;

        timer_add(scheduler, fiber, fiber.deadline);
    }

    reason: s32;
    waiter: Waiter;

    waiter.slot = *reason;

    error := wait_on(scheduler, fiber, *half.readers, *waiter);

    if error != .NONE return error;

    if !waiter.completed {
        // The deadline's timer woke the fiber (and took it off the wait list).
        waiter_unlink(*waiter);

        return .TIMED_OUT;
    }

    if reason == 1 return .CLOSED;

    return .NONE;
}
