// Timers and cancellation: sleeps end in deadline order without blocking other fibers, and
// cancel() ends a fiber's blocking call at once, whatever it waits on.
#import "Basic";
Io :: #import "Extensions/Io";

wake_order: [..] int;
// When each sleep was due, in the order the sleepers woke. A slow interpreter can take a while to
// start ten fibers, so the order is checked against these, not against the delays alone.
wake_deadlines: [..] Apollo_Time;
group: Io.Wait_Group;

sleeper :: (milliseconds: int) {
    deadline := current_time_monotonic() + milliseconds_to_apollo(milliseconds);
    error := Io.sleep(milliseconds);

    assert(error == .NONE);
    array_add(*wake_order, milliseconds);
    array_add(*wake_deadlines, deadline);
    Io.done(*group);
}

test_sleep_order :: () {
    started := current_time_monotonic();

    // Spawned out of order; they must wake in order of their deadlines.
    delays := int.[60, 20, 100, 40, 80, 10, 30, 70, 50, 90];

    Io.add(*group, delays.count);
    for delays Io.spawn(sleeper, it);

    // The same deadline wakes in the order the sleeps began.
    Io.add(*group, 3);
    for 1..3 Io.spawn(equal_sleeper, it);

    Io.wait(*group);

    elapsed := to_milliseconds(current_time_monotonic() - started);

    assert(elapsed >= 100, "everything ended after % ms", elapsed);
    assert(elapsed < 1500, "sleeping fibers overlap: % ms", elapsed);

    for 1..wake_deadlines.count - 1 {
        if wake_deadlines[it] < wake_deadlines[it - 1] {
            assert(false, "woke out of order: % ms after % ms", wake_order[it], wake_order[it - 1]);
        }
    }

    assert(wake_order.count == delays.count + 3);
    assert(equal_order.count == 3 && equal_order[0] == 1 && equal_order[1] == 2 && equal_order[2] == 3, "same-deadline order");

    // sleep(0) only yields.
    assert(Io.sleep(0) == .NONE);
    assert(Io.sleep(-5) == .NONE);
}

equal_order: [..] int;

equal_sleeper :: (index: int) {
    deadline := current_time_monotonic() + milliseconds_to_apollo(25);

    Io.sleep(25);
    array_add(*equal_order, index);
    array_add(*wake_order, 25);
    array_add(*wake_deadlines, deadline);
    Io.done(*group);
}

// A busy fiber is not held up by sleepers, and sleepers are not starved by it.
ticks: int;

busy :: () {
    while !Io.is_canceled() {
        ticks += 1;
        Io.yield();
    }
}

test_sleep_while_busy :: () {
    handle := Io.spawn(busy);

    assert(Io.sleep(30) == .NONE);
    assert(ticks > 10, "the busy fiber ran meanwhile: % ticks", ticks);

    Io.cancel(handle);
    Io.sleep(1);
}

// Cancellation.
results: [8] Io.Io_Error;
reached: [8] bool;

Blocked :: struct {
    index: int;
    channel: *Io.Channel(int);
    mutex: *Io.Fiber_Mutex;
    group: *Io.Wait_Group;
}

block_in_sleep :: (job: Blocked) {
    results[job.index] = Io.sleep(100_000);
    reached[job.index] = true;
}

block_in_recv :: (job: Blocked) {
    _, error := Io.recv(job.channel);
    results[job.index] = error;
    reached[job.index] = true;
}

block_in_send :: (job: Blocked) {
    results[job.index] = Io.send(job.channel, 1);
    reached[job.index] = true;
}

block_in_lock :: (job: Blocked) {
    results[job.index] = Io.lock(job.mutex);
    reached[job.index] = true;
}

block_in_wait :: (job: Blocked) {
    results[job.index] = Io.wait(job.group);
    reached[job.index] = true;
}

// Cancel before the fiber ever ran: every blocking call returns .CANCELED from then on.
late_start :: (job: Blocked) {
    results[job.index] = Io.sleep(100_000);
    reached[job.index] = true;
    // Still canceled afterwards.
    results[job.index + 1] = Io.yield();
    reached[job.index + 1] = true;
}

test_cancel :: () {
    unbuffered := Io.make_channel(int);
    full := Io.make_channel(int, 1);

    Io.try_send(full, 0);

    mutex: Io.Fiber_Mutex;

    Io.lock(*mutex);
    never: Io.Wait_Group;

    Io.add(*never, 1);

    handles: [5] Io.Fiber_Handle;
    handles[0] = Io.spawn(block_in_sleep, Blocked.{ index = 0 });
    handles[1] = Io.spawn(block_in_recv, Blocked.{ index = 1, channel = unbuffered });
    handles[2] = Io.spawn(block_in_send, Blocked.{ index = 2, channel = full });
    handles[3] = Io.spawn(block_in_lock, Blocked.{ index = 3, mutex = *mutex });
    handles[4] = Io.spawn(block_in_wait, Blocked.{ index = 4, group = *never });

    // Let all five reach their blocking call.
    for 1..3 Io.yield();
    for reached assert(!it, "still blocked");

    for handles Io.cancel(it);
    for 1..3 Io.yield();

    for 0..4 {
        assert(reached[it], "fiber % did not return", it);
        assert(results[it] == .CANCELED, "fiber % got %", it, results[it]);
    }

    // A canceled receiver was taken off the channel's wait list: this value is not lost to it.
    assert(!Io.try_send(unbuffered, 1), "nobody is waiting any more");

    // The mutex still belongs to us, and a canceled waiter did not take it.
    assert(!Io.try_lock(*mutex));
    Io.unlock(*mutex);
    assert(Io.try_lock(*mutex));
    Io.unlock(*mutex);

    // Canceled before it first ran.
    late := Io.spawn(late_start, Blocked.{ index = 5 });

    Io.cancel(late);
    for 1..3 Io.yield();

    assert(reached[5] && results[5] == .CANCELED);
    assert(reached[6] && results[6] == .CANCELED);

    // A handle to a fiber that ended does nothing, even after its record is reused.
    Io.cancel(late);
    other := Io.spawn(block_in_sleep, Blocked.{ index = 7 });

    Io.cancel(late);
    for 1..3 Io.yield();

    assert(!reached[7], "a stale handle canceled somebody else");
    Io.cancel(other);
    for 1..3 Io.yield();

    assert(reached[7] && results[7] == .CANCELED);

    Io.free_channel(unbuffered);
    Io.free_channel(full);
}

// A fiber cancels itself, and a fiber can see that it was canceled.
self_result: Io.Io_Error;

self_cancel :: () {
    Io.cancel(Io.current());
    assert(Io.is_canceled());
    self_result = Io.sleep(10_000);
}

test_self_cancel :: () {
    Io.spawn(self_cancel);
    for 1..3 Io.yield();
    assert(self_result == .CANCELED);
    assert(!Io.is_canceled(), "this fiber was not canceled");
}

// Fibers still running when io_run's first fiber returns are canceled and finish.
leftovers_finished: int;

leftover :: () {
    error := Io.sleep(1_000_000);
    assert(error == .CANCELED);
    leftovers_finished += 1;
}

root :: () {
    test_sleep_order();
    test_sleep_while_busy();
    test_cancel();
    test_self_cancel();

    for 1..5 Io.spawn(leftover);

    Io.yield();
}

main :: () {
    Io.io_run(root);
    assert(leftovers_finished == 5, "leftovers ended: %", leftovers_finished);
    print("ok\n");
}
