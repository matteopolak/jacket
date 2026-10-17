// Coordination between fibers: channels, wait groups and mutexes. They only ever block the
// calling fiber. Everything here assumes the fibers share one thread (one `io_run`), which is
// what makes plain loads and stores safe.

// A bounded queue of `T` between fibers. `send` blocks while it is full, `recv` while it is empty.
// With capacity 0 a send and a receive meet: each waits for the other.
Channel :: struct(T: Type) {
    buffer: *T;
    capacity: s64;
    head: s64;
    count: s64;
    closed: bool;
    senders: Waiter;
    receivers: Waiter;
    allocator: Allocator;
}

// A channel that holds up to `capacity` values, made with `context.allocator`.
make_channel :: ($T: Type, capacity := 0) -> *Channel(T) {
    assert(capacity >= 0, "Io: a channel's capacity cannot be negative");

    channel := New(Channel(T));
    channel.capacity = capacity;
    channel.allocator = context.allocator;
    if capacity > 0 channel.buffer = cast(*T) alloc(capacity * size_of(T));

    return channel;
}

// Frees a channel from `make_channel`. No fiber may be waiting on it.
free_channel :: (channel: *Channel($T)) {
    assert(waiters_empty(*channel.senders) && waiters_empty(*channel.receivers), "Io: a channel was freed while fibers wait on it");

    if channel.buffer release_owned(channel.buffer, channel.allocator);
    release_owned(channel, channel.allocator);
} @Releases

// Puts `value` into the channel, blocking while it is full. `.CLOSED` if the channel is (or becomes)
// closed, `.CANCELED` if the fiber is canceled while it waits.
send :: (channel: *Channel($T), value: T) -> Io_Error {
    fiber, scheduler := current_fiber();
    if fiber.canceled return .CANCELED;
    if channel.closed return .CLOSED;

    if hand_to_receiver(channel, value) return .NONE;

    if channel.count < channel.capacity {
        channel.buffer[(channel.head + channel.count) % channel.capacity] = value;
        channel.count += 1;

        return .NONE;
    }

    waiter: Waiter;
    waiter.slot = *value;

    error := wait_on(scheduler, fiber, *channel.senders, *waiter);
    if error != .NONE return error;
    if !waiter.completed return .CLOSED;

    return .NONE;
}

// A `send` that gives up instead of blocking: whether the value was taken.
try_send :: (channel: *Channel($T), value: T) -> bool {
    if channel.closed return false;
    if hand_to_receiver(channel, value) return true;
    if channel.count >= channel.capacity return false;

    channel.buffer[(channel.head + channel.count) % channel.capacity] = value;
    channel.count += 1;

    return true;
}

// Takes the oldest value, blocking while the channel is empty. `.CLOSED` once it is closed and
// drained, `.CANCELED` if the fiber is canceled while it waits.
recv :: (channel: *Channel($T)) -> T, Io_Error {
    result: T;
    fiber, scheduler := current_fiber();

    if fiber.canceled return result, .CANCELED;

    if take_ready(channel, *result) return result, .NONE;
    if channel.closed return result, .CLOSED;

    waiter: Waiter;
    waiter.slot = *result;

    error := wait_on(scheduler, fiber, *channel.receivers, *waiter);
    if error != .NONE return result, error;
    if !waiter.completed return result, .CLOSED;

    return result, .NONE;
}

// A `recv` that gives up instead of blocking: the value and whether there was one.
try_recv :: (channel: *Channel($T)) -> T, bool {
    result: T;
    found := take_ready(channel, *result);

    return result, found;
}

// No more values will be sent. Fibers waiting to send get `.CLOSED`; receivers get what is
// buffered, then `.CLOSED`.
close :: (channel: *Channel($T)) {
    if channel.closed return;

    channel.closed = true;

    while true {
        waiter := waiter_pop(*channel.receivers);
        if !waiter break;

        wake(waiter.fiber);
    }

    while true {
        waiter := waiter_pop(*channel.senders);
        if !waiter break;

        wake(waiter.fiber);
    }
} @Releases

// Waits for a number of tasks, as many as were `add`ed and have not yet been `done`.
Wait_Group :: struct {
    count: s64;
    waiters: Waiter;
}

add :: (group: *Wait_Group, count := 1) {
    group.count += count;
    assert(group.count >= 0, "Io: a Wait_Group's count went negative");

    if group.count == 0 wake_all(*group.waiters);
}

done :: (group: *Wait_Group) {
    add(group, -1);
}

// Blocks until the count is back to zero.
wait :: (group: *Wait_Group) -> Io_Error {
    fiber, scheduler := current_fiber();
    if fiber.canceled return .CANCELED;
    if group.count == 0 return .NONE;

    waiter: Waiter;

    return wait_on(scheduler, fiber, *group.waiters, *waiter);
}

// A lock for fibers. A fiber that cannot take it waits without blocking the thread, so unlike
// `Thread`'s `Mutex` it is safe to hold across a `sleep` or a channel operation. It is not
// recursive, and waiters get it in the order they asked.
Fiber_Mutex :: struct {
    locked: bool;
    waiters: Waiter;
}

lock :: (mutex: *Fiber_Mutex) -> Io_Error {
    fiber, scheduler := current_fiber();
    if fiber.canceled return .CANCELED;

    if !mutex.locked {
        mutex.locked = true;

        return .NONE;
    }

    waiter: Waiter;

    // `unlock` hands the lock over, so a woken waiter already holds it.
    return wait_on(scheduler, fiber, *mutex.waiters, *waiter);
}

try_lock :: (mutex: *Fiber_Mutex) -> bool {
    if mutex.locked return false;

    mutex.locked = true;

    return true;
}

unlock :: (mutex: *Fiber_Mutex) {
    assert(mutex.locked, "Io: unlock of a Fiber_Mutex that is not locked");

    waiter := waiter_pop(*mutex.waiters);
    if waiter wake(waiter.fiber);
    else mutex.locked = false;
}

#scope_module

// A fiber waiting on a list, kept on its own stack. A list is a Waiter too: the head of a
// circular chain that starts out zeroed and is set up on first use.
Waiter :: struct {
    fiber: *Fiber;
    previous: *Waiter;
    next: *Waiter;
    // Where a channel operation reads or writes its value.
    slot: *void;
    // Set when the operation took place (as opposed to the wait ending because of a close).
    completed: bool;
}

waiters_empty :: (list: *Waiter) -> bool {
    return list.next == null || list.next == list;
}

waiter_push :: (list: *Waiter, waiter: *Waiter) {
    if !list.next {
        list.next = list;
        list.previous = list;
    }

    waiter.previous = list.previous;
    waiter.next = list;
    list.previous.next = waiter;
    list.previous = waiter;
}

waiter_unlink :: (waiter: *Waiter) {
    if !waiter.next return;

    waiter.previous.next = waiter.next;
    waiter.next.previous = waiter.previous;
    waiter.next = null;
    waiter.previous = null;
}

waiter_pop :: (list: *Waiter) -> *Waiter {
    if waiters_empty(list) return null;

    waiter := list.next;
    waiter_unlink(waiter);

    return waiter;
}

wake_all :: (list: *Waiter) {
    while true {
        waiter := waiter_pop(list);
        if !waiter break;

        wake(waiter.fiber);
    }
}

// Queues `waiter` on `list` and parks the fiber until it is woken or canceled.
wait_on :: (scheduler: *Scheduler, fiber: *Fiber, list: *Waiter, waiter: *Waiter) -> Io_Error {
    waiter.fiber = fiber;
    waiter_push(list, waiter);
    fiber.waiter = waiter;

    error := park(scheduler, fiber);

    fiber.waiter = null;

    return error;
}

// A waiting receiver takes `value` directly.
hand_to_receiver :: (channel: *Channel($T), value: T) -> bool {
    waiter := waiter_pop(*channel.receivers);
    if !waiter return false;

    (cast(*T) waiter.slot).* = value;
    waiter.completed = true;
    wake(waiter.fiber);

    return true;
}

// Takes a value that is available now: the oldest buffered one (letting a waiting sender refill
// the buffer), or a waiting sender's own.
take_ready :: (channel: *Channel($T), result: *T) -> bool {
    if channel.count > 0 {
        result.* = channel.buffer[channel.head];
        channel.head = (channel.head + 1) % channel.capacity;
        channel.count -= 1;

        sender := waiter_pop(*channel.senders);
        if sender {
            channel.buffer[(channel.head + channel.count) % channel.capacity] = (cast(*T) sender.slot).*;
            channel.count += 1;
            sender.completed = true;
            wake(sender.fiber);
        }

        return true;
    }

    sender := waiter_pop(*channel.senders);
    if !sender return false;

    result.* = (cast(*T) sender.slot).*;
    sender.completed = true;
    wake(sender.fiber);

    return true;
}
