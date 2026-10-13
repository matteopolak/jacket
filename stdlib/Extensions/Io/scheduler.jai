// The fiber scheduler: one per OS thread, started by `io_run`.
//
// Fibers do not switch directly to each other. The scheduler runs on the thread's own stack and
// resumes one ready fiber at a time; a fiber that blocks, yields or ends switches back to it.
// Blocking is always the same three steps: put the fiber on whatever will wake it (a timer, a
// channel's wait list, ...), `park` it, and report what woke it. `wake` makes a parked fiber ready
// again, `cancel` wakes it with `.CANCELED`.

// Why a blocking call returned early or failed. `platform_code()` gives the system's own error
// number for the last failure of the running fiber, which is how `OTHER` stays useful.
Io_Error :: enum u8 {
    NONE;
    // `cancel` was called for this fiber, now or earlier.
    CANCELED;
    // The channel was closed (and, for a receive, is empty), or the handle was closed while the
    // fiber waited on it.
    CLOSED;
    // The `with_deadline` limit passed.
    TIMED_OUT;
    // `read_full` hit the end of the stream before it had everything.
    END_OF_FILE;
    CONNECTION_RESET;
    CONNECTION_REFUSED;
    CONNECTION_ABORTED;
    BROKEN_PIPE;
    NOT_CONNECTED;
    ADDRESS_IN_USE;
    ADDRESS_UNAVAILABLE;
    NETWORK_UNREACHABLE;
    HOST_UNREACHABLE;
    NOT_FOUND;
    ALREADY_EXISTS;
    PERMISSION_DENIED;
    INVALID;
    NO_SPACE;
    TOO_MANY_OPEN_FILES;
    IS_A_DIRECTORY;
    // This system has no implementation yet (IO on Windows), or the operation does not apply.
    UNSUPPORTED;
    // The name could not be resolved.
    RESOLVE_FAILED;
    // Anything else; see `platform_code()`.
    OTHER;
}

Spawn_Entry :: #type (data: *void);

Fiber_State :: enum u8 {
    // Waiting in the pool for `spawn` to reuse it.
    FREE;
    // In the ready queue.
    READY;
    RUNNING;
    // Parked: waiting for a timer, a channel, a mutex...
    BLOCKED;
    // Its entry procedure finished.
    DEAD;
}

// One fiber. Treat it as opaque: use `Fiber_Handle` to refer to one from outside the scheduler.
Fiber :: struct {
    ctx: Fiber_Context;
    stack: Stack;
    state: Fiber_State;
    // Set by `cancel`; makes every blocking call return `.CANCELED`.
    canceled: bool;
    scheduler: *Scheduler;
    id: u64;
    // Bumped each time the fiber ends, so a stale `Fiber_Handle` is recognised.
    generation: u64;
    // Ready queue and pool links.
    next: *Fiber;
    // All live fibers.
    all_previous: *Fiber;
    all_next: *Fiber;
    entry: Spawn_Entry;
    data: *void;
    // What `park` returns when the fiber is woken.
    wake_reason: Io_Error;
    // What the fiber is queued on while it is blocked, to take it off again when it is canceled.
    waiter: *Waiter;
    timer_slot: s64 = -1;
    // The engine operation this fiber is blocked in, and the name lookup, if any.
    op: *Op;
    job: *Resolve_Job;
    // Absolute `now_nanoseconds()` limit for every IO call (0: none), set by `with_deadline`.
    deadline: s64;
    // The system's error number behind the last failed IO call.
    platform_code: s32;
    fiber_context: #Context;
    temporary_storage: Temporary_Storage;
    temporary_ready: bool;
}

Fiber_Handle :: struct {
    fiber: *Fiber;
    generation: u64;
}

Timer :: struct {
    deadline: s64;
    sequence: u64;
    fiber: *Fiber;
}

Scheduler :: struct {
    // The thread's own stack, which the scheduler loop runs on.
    ctx: Fiber_Context;
    thread_index: u32;
    allocator: Allocator;
    current: *Fiber;
    ready_head: *Fiber;
    ready_tail: *Fiber;
    // Ended fibers waiting for `spawn` to reuse them, and how many of those still hold a stack.
    pool: *Fiber;
    pooled_stacks: s64;
    all: *Fiber;
    // Fibers that have not ended.
    live: s64;
    timers: [..] Timer;
    timer_sequence: u64;
    next_id: u64;
    root: *Fiber;
    root_done: bool;
    draining: bool;
    switches: u64;
    // Operations the completion engine has in flight, and name lookups under way; a scheduler
    // with none, no timers and nothing ready has nothing left to wait for.
    pending: s64;
    engine: *Engine;
    // Helper threads and the pipe they wake the scheduler through (resolver.jai).
    waker: *Wake_State;
    // What libraries keep per scheduler (`scheduler_local`), dropped when the scheduler ends.
    locals: [..] Local_Entry;
    // The alternate signal stack the stack overflow report runs on (overflow.jai).
    #if OS != .WINDOWS {
        alt_memory: *void;
        alt_previous: POSIX.stack_t;
    }
}

#add_context io_scheduler: *Scheduler;

// Runs `proc` as the first fiber of a new scheduler on this thread and returns when it has
// returned. Fibers it started that are still alive then are canceled, and run until they end.
io_run :: (proc: $P) {
    run_scheduler(spawn_block(proc));
}

io_run :: (proc: $P, data: $T) {
    run_scheduler(spawn_block(proc, data));
}

// Starts `proc` as a new fiber on the current scheduler; it first runs when the caller blocks or
// yields. It gets a copy of the caller's `context` (with temporary storage of its own) and
// `data` is copied for it.
spawn :: (proc: $P) -> Fiber_Handle {
    return start_fiber(current_scheduler(), spawn_block(proc));
}

spawn :: (proc: $P, data: $T) -> Fiber_Handle {
    return start_fiber(current_scheduler(), spawn_block(proc, data));
}

// Lets the other ready fibers run first. `.CANCELED` if this fiber was canceled.
yield :: () -> Io_Error {
    fiber, scheduler := current_fiber();
    if fiber.canceled return .CANCELED;

    fiber.state = .READY;
    push_ready(scheduler, fiber);
    fiber_switch(*fiber.ctx, *scheduler.ctx);

    return ifx fiber.canceled then Io_Error.CANCELED else .NONE;
}

// Blocks this fiber, not the thread, for at least `milliseconds`. Ends early with `.TIMED_OUT`
// when a `with_deadline` limit comes first.
sleep :: (milliseconds: s64) -> Io_Error {
    fiber, scheduler := current_fiber();
    if fiber.canceled return .CANCELED;
    if milliseconds <= 0 return yield();

    end := now_nanoseconds() + milliseconds * 1_000_000;
    limited := fiber.deadline != 0 && fiber.deadline <= end;

    if limited end = fiber.deadline;

    timer_add(scheduler, fiber, end);

    error := park(scheduler, fiber);
    if error == .NONE && limited return .TIMED_OUT;

    return error;
}

// Makes the blocking calls of `fiber` return `.CANCELED`, now and from now on. The call it is
// blocked in, if any, returns at once. Does nothing for a fiber that has ended.
cancel :: (handle: Fiber_Handle) {
    fiber := handle.fiber;
    if !fiber || fiber.generation != handle.generation || fiber.state == .FREE || fiber.state == .DEAD return;

    assert(fiber.scheduler.thread_index == context.thread_index, "Io: a fiber can only be canceled from its own thread");
    cancel_fiber(fiber);
}

// Whether the running fiber was canceled, for loops that do not block.
is_canceled :: () -> bool {
    fiber := current_fiber();

    return fiber.canceled;
}

// Whether the calling code runs on a fiber of a scheduler, that is inside `io_run`.
in_scheduler :: () -> bool {
    scheduler := context.io_scheduler;

    return scheduler != null && scheduler.thread_index == context.thread_index && scheduler.current != null;
}

// The handle of the running fiber.
current :: () -> Fiber_Handle {
    fiber := current_fiber();

    return .{ fiber, fiber.generation };
}

#scope_module

// A procedure and its argument, copied to the heap for the new fiber to take.
spawn_block :: (proc: $P) -> Spawn_Block {
    Block :: struct {
        proc: P;
    }

    block := New(Block);
    block.proc = proc;

    thunk :: (raw: *void) {
        block := cast(*Block) raw;
        proc := block.proc;
        free(block);

        proc();
    }

    return .{ thunk, block };
}

spawn_block :: (proc: $P, data: $T) -> Spawn_Block {
    Block :: struct {
        proc: P;
        data: T;
    }

    block := New(Block);
    block.proc = proc;
    block.data = data;

    thunk :: (raw: *void) {
        block := cast(*Block) raw;
        proc := block.proc;
        data := block.data;

        free(block);

        proc(data);
    }

    return .{ thunk, block };
}

Spawn_Block :: struct {
    entry: Spawn_Entry;
    data: *void;
}

current_scheduler :: () -> *Scheduler {
    scheduler := context.io_scheduler;
    assert(scheduler != null && scheduler.thread_index == context.thread_index && scheduler.current != null, "Io: this must run on a fiber, inside io_run");

    return scheduler;
}

current_fiber :: () -> *Fiber, *Scheduler {
    scheduler := current_scheduler();

    return scheduler.current, scheduler;
}

now_nanoseconds :: () -> s64 {
    nanoseconds := to_nanoseconds(current_time_monotonic());

    return nanoseconds;
}

// ---------------------------------------------------------------------------------------------
// Starting and ending fibers

start_fiber :: (scheduler: *Scheduler, block: Spawn_Block) -> Fiber_Handle {
    fiber := scheduler.pool;
    if fiber {
        scheduler.pool = fiber.next;
        if fiber.stack.base scheduler.pooled_stacks -= 1;
    } else {
        fiber = New(Fiber, allocator = scheduler.allocator);
        fiber.scheduler = scheduler;
    }

    if !fiber.stack.base {
        fiber.stack = stack_allocate(STACK_BYTES);
        assert(fiber.stack.base != null, "Io: could not allocate a fiber stack");
    }

    if !fiber.temporary_ready {
        init_temporary_storage(*fiber.temporary_storage, TEMPORARY_BYTES, null, scheduler.allocator);
        fiber.temporary_ready = true;
    }

    scheduler.next_id += 1;
    fiber.id = scheduler.next_id;
    fiber.canceled = false;
    fiber.wake_reason = .NONE;
    fiber.waiter = null;
    fiber.timer_slot = -1;
    fiber.op = null;
    fiber.job = null;
    fiber.deadline = 0;
    fiber.platform_code = 0;
    fiber.entry = block.entry;
    fiber.data = block.data;

    // A copy of the spawner's context, as a new thread gets: with its own temporary storage and
    // stack trace.
    fiber.fiber_context = context;
    fiber.fiber_context.temporary_storage = *fiber.temporary_storage;
    fiber.fiber_context.stack_trace = null;
    fiber.fiber_context.io_scheduler = scheduler;

    fiber_init(*fiber.ctx, fiber.stack.base, fiber.stack.size, fiber_main, fiber);

    fiber.all_previous = null;
    fiber.all_next = scheduler.all;
    if scheduler.all scheduler.all.all_previous = fiber;

    scheduler.all = fiber;
    scheduler.live += 1;

    fiber.state = .READY;
    push_ready(scheduler, fiber);

    return .{ fiber, fiber.generation };
}

// The first code of every fiber, on its own stack and without a context.
fiber_main :: (raw: *void) #c_call {
    fiber := cast(*Fiber) raw;

    push_context fiber.fiber_context {
        fiber.entry(fiber.data);
        reset_temporary_storage();
    }

    fiber.state = .DEAD;
    fiber_switch(*fiber.ctx, *fiber.scheduler.ctx);

    // Nothing switches to a fiber that ended.
    while true {}
}

// The scheduler's side of a fiber that ended: take it out of the live list and keep it for reuse.
retire_fiber :: (scheduler: *Scheduler, fiber: *Fiber) {
    if fiber.all_previous fiber.all_previous.all_next = fiber.all_next;
    else scheduler.all = fiber.all_next;
    if fiber.all_next fiber.all_next.all_previous = fiber.all_previous;

    if fiber == scheduler.root scheduler.root_done = true;

    scheduler.live -= 1;
    fiber.generation += 1;

    // A handle to this fiber may still exist, so the record stays; only a stack beyond the pool
    // limit goes back to the OS.
    if scheduler.pooled_stacks >= POOL_LIMIT release_stack(scheduler, fiber);
    else scheduler.pooled_stacks += 1;

    fiber.state = .FREE;
    fiber.next = scheduler.pool;
    scheduler.pool = fiber;
}

release_stack :: (scheduler: *Scheduler, fiber: *Fiber) {
    fiber_free(*fiber.ctx);
    stack_release(fiber.stack);
    fiber.stack = .{};

    if fiber.temporary_ready {
        deinit_temporary_storage(*fiber.temporary_storage, scheduler.allocator, true);
        fiber.temporary_ready = false;
    }
}

// ---------------------------------------------------------------------------------------------
// Ready queue, parking and waking

push_ready :: (scheduler: *Scheduler, fiber: *Fiber) {
    fiber.next = null;
    if scheduler.ready_tail scheduler.ready_tail.next = fiber;
    else scheduler.ready_head = fiber;
    scheduler.ready_tail = fiber;
}

pop_ready :: (scheduler: *Scheduler) -> *Fiber {
    fiber := scheduler.ready_head;
    if !fiber return null;

    scheduler.ready_head = fiber.next;
    if !scheduler.ready_head scheduler.ready_tail = null;

    fiber.next = null;

    return fiber;
}

// Suspends the running fiber until something calls `wake` (or `cancel`) for it, and says which.
park :: (scheduler: *Scheduler, fiber: *Fiber) -> Io_Error {
    fiber.state = .BLOCKED;
    fiber.wake_reason = .NONE;
    fiber_switch(*fiber.ctx, *scheduler.ctx);

    return fiber.wake_reason;
}

wake :: (fiber: *Fiber, reason := Io_Error.NONE) {
    assert(fiber.state == .BLOCKED, "Io: woke a fiber that is not blocked");

    if fiber.timer_slot >= 0 timer_remove(fiber.scheduler, fiber.timer_slot);

    fiber.wake_reason = reason;
    fiber.state = .READY;
    push_ready(fiber.scheduler, fiber);
}

cancel_fiber :: (fiber: *Fiber) {
    if fiber.canceled return;

    fiber.canceled = true;
    if fiber.state != .BLOCKED return;

    // An operation in the engine owns its buffer until it completes, so the fiber stays blocked
    // until the engine reports the canceled operation (see `cancel_op`).
    if fiber.op {
        cancel_op(fiber, .CANCELED);

        return;
    }

    if fiber.job {
        abandon_job(fiber);
        wake(fiber, .CANCELED);

        return;
    }

    if fiber.waiter {
        waiter_unlink(fiber.waiter);
        fiber.waiter = null;
    }

    if fiber.timer_slot >= 0 timer_remove(fiber.scheduler, fiber.timer_slot);

    wake(fiber, .CANCELED);
}

// ---------------------------------------------------------------------------------------------
// Timers: a binary heap ordered by deadline, then by creation order. A fiber knows its slot, so
// a canceled sleep leaves the heap at once.

timer_less :: (a: Timer, b: Timer) -> bool {
    if a.deadline != b.deadline return a.deadline < b.deadline;

    return a.sequence < b.sequence;
}

timer_place :: (scheduler: *Scheduler, slot: s64, timer: Timer) {
    scheduler.timers[slot] = timer;
    timer.fiber.timer_slot = slot;
}

timer_up :: (scheduler: *Scheduler, slot: s64) {
    timer := scheduler.timers[slot];

    while slot > 0 {
        parent := (slot - 1) / 2;
        if !timer_less(timer, scheduler.timers[parent]) break;

        timer_place(scheduler, slot, scheduler.timers[parent]);
        slot = parent;
    }

    timer_place(scheduler, slot, timer);
}

timer_down :: (scheduler: *Scheduler, slot: s64) {
    timer := scheduler.timers[slot];
    count := scheduler.timers.count;

    while true {
        child := slot * 2 + 1;
        if child >= count break;
        if child + 1 < count && timer_less(scheduler.timers[child + 1], scheduler.timers[child]) child += 1;
        if !timer_less(scheduler.timers[child], timer) break;

        timer_place(scheduler, slot, scheduler.timers[child]);
        slot = child;
    }

    timer_place(scheduler, slot, timer);
}

timer_add :: (scheduler: *Scheduler, fiber: *Fiber, deadline: s64) {
    scheduler.timer_sequence += 1;

    array_add(*scheduler.timers, .{ deadline, scheduler.timer_sequence, fiber });
    timer_up(scheduler, scheduler.timers.count - 1);
}

timer_remove :: (scheduler: *Scheduler, slot: s64) {
    removed := scheduler.timers[slot];
    removed.fiber.timer_slot = -1;

    last := scheduler.timers[scheduler.timers.count - 1];

    scheduler.timers.count -= 1;
    if slot == scheduler.timers.count return;

    timer_place(scheduler, slot, last);
    timer_up(scheduler, slot);
    timer_down(scheduler, last.fiber.timer_slot);
}

// Wakes every fiber whose sleep is over.
fire_timers :: (scheduler: *Scheduler) {
    if scheduler.timers.count == 0 return;

    now := now_nanoseconds();
    while scheduler.timers.count > 0 && scheduler.timers[0].deadline <= now {
        fiber := scheduler.timers[0].fiber;

        timer_remove(scheduler, 0);

        // A limit that passes while the fiber waits for the engine or a lookup ends that call only.
        if fiber.op {
            cancel_op(fiber, .TIMED_OUT);
        } else if fiber.job {
            abandon_job(fiber);
            wake(fiber, .TIMED_OUT);
        } else {
            // A fiber queued on a list (a pipe read with a limit) leaves it now: until it runs, a
            // producer must not find it there and wake it a second time.
            if fiber.waiter {
                waiter_unlink(fiber.waiter);
                fiber.waiter = null;
            }

            wake(fiber);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The scheduler loop

run_scheduler :: (block: Spawn_Block) {
    previous := context.io_scheduler;

    assert(previous == null || previous.thread_index != context.thread_index, "Io: io_run cannot be nested on one thread");

    scheduler := New(Scheduler);
    scheduler.thread_index = context.thread_index;
    scheduler.allocator = context.allocator;

    guard_overflow(scheduler);
    engine_start(scheduler);

    handle := start_fiber(scheduler, block);

    scheduler.root = handle.fiber;

    schedule(scheduler);

    drop_locals(scheduler);
    engine_stop(scheduler);
    unguard_overflow(scheduler);

    pooled := scheduler.pool;
    while pooled {
        next := pooled.next;
        release_stack(scheduler, pooled);
        free(pooled, scheduler.allocator);
        pooled = next;
    }

    array_reset(*scheduler.timers);
    array_reset(*scheduler.locals);
    free(scheduler);
}

schedule :: (scheduler: *Scheduler) {
    while true {
        if scheduler.root_done && !scheduler.draining {
            scheduler.draining = true;
            cancel_all(scheduler);
        }

        if scheduler.live == 0 break;

        // Timers are read from the clock when there is nothing else to do, and now and then
        // while fibers keep each other busy.
        if scheduler.ready_head == null || scheduler.switches % 64 == 0 fire_timers(scheduler);

        // Fibers that keep each other busy still see their IO complete now and then.
        if scheduler.ready_head != null && scheduler.pending > 0 && scheduler.switches % 64 == 0 engine_wait(scheduler, 0);

        fiber := pop_ready(scheduler);
        if !fiber {
            wait_step(scheduler);
            continue;
        }

        scheduler.switches += 1;
        scheduler.current = fiber;
        fiber.state = .RUNNING;
        fiber_switch(*scheduler.ctx, *fiber.ctx);
        scheduler.current = null;

        if fiber.state == .DEAD retire_fiber(scheduler, fiber);
    }
}

cancel_all :: (scheduler: *Scheduler) {
    fiber := scheduler.all;
    while fiber {
        cancel_fiber(fiber);
        fiber = fiber.all_next;
    }
}

// ---------------------------------------------------------------------------------------------
// The wait step: what the scheduler does when no fiber is ready. It hands the operations the
// fibers queued to the engine, waits for completions for at most the time to the next timer
// (forever when there is none), and lets the engine `wake` the fibers they belong to.

wait_step :: (scheduler: *Scheduler) {
    if scheduler.ready_head return;

    timeout := -1;
    if scheduler.timers.count > 0 timeout = max(scheduler.timers[0].deadline - now_nanoseconds(), 0);

    if timeout < 0 && scheduler.pending == 0 {
        print("Io: deadlock: every fiber is blocked and nothing can wake them\n", to_standard_error = true);
        exit(1);
    }

    engine_wait(scheduler, timeout);
    fire_timers(scheduler);
}
