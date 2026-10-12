// The file thread pool of the readiness backends (epoll, kqueue).
//
// A regular file never reports "not ready", so a read or write on one cannot wait in the poller
// the way a socket does; run inline it would stall every fiber of the scheduler's thread for as long
// as the disk takes. Reads and writes longer than `Engine.file_inline` bytes therefore go to a pool
// of worker threads that make the blocking `pread`/`pwrite` call, and the completion comes back to
// the scheduler that asked. Shorter ones run inline: a short read nearly always hits the page
// cache and finishes in a couple of microseconds, which is far less than the hand-off to a worker
// and back (docs/stdlib/io-engine.md has the numbers). io_uring and the Windows backends do not use
// the pool; they have native file operations.
//
// Shape:
//   - One pool per process (`file_pool`), shared by every scheduler, started when the first
//     scheduler needs it and ended when the last one is done (so a program that runs `io_run` again
//     later starts it again, and nothing is left running at exit). It has `file_threads` workers
//     (default DEFAULT_FILE_THREADS, `JAI_IO_FILE_THREADS`; 0 turns the pool off) and one queue.
//   - Every scheduler that uses the pool has a `Pool_Link`: a list of finished operations under a
//     mutex, and the means to wake the scheduler's wait. A worker appends the operation and
//     wakes while holding that mutex; the scheduler takes the list under it. The scheduler also
//     takes the mutex once before it frees the link, so a worker never touches freed memory.
//   - The wake-up is an eventfd on epoll, read by an ordinary internal READ operation that is armed
//     while operations are out (the engine waits on it like on any descriptor), and an
//     EVFILT_USER event on kqueue.
//   - A pool operation cannot be interrupted (`Op.uncancelable`): a cancel or deadline waits for
//     it to finish, then the fiber wakes with the result. Closing a file waits for the operations
//     still out on any file of the scheduler.

// The workers started, at most.
POOL_MAX_THREADS :: 16;

// Defaults of `JAI_IO_FILE_THREADS` and `JAI_IO_FILE_INLINE`.
DEFAULT_FILE_THREADS :: 4;
DEFAULT_FILE_INLINE :: 32768;

// A scheduler's end of the pool.
Pool_Link :: struct {
    mutex: Thread.Mutex;
    done_head: *Op;
    done_tail: *Op;
    // Operations handed to workers whose completion the scheduler has not taken yet.
    inflight: s64;

    // epoll: the eventfd, and the internal READ that waits for it.
    wake_fd: s64 = -1;
    wake_op: *Op;
    wake_buffer: [8] u8;
    // kqueue: the descriptor to trigger.
    queue_fd: s64 = -1;
}

File_Pool :: struct {
    mutex: Thread.Mutex;
    work: Thread.Semaphore;
    head: *Op;
    tail: *Op;
    workers: [POOL_MAX_THREADS] Thread.Thread;
    started: s64;
    // Schedulers using it.
    users: s64;
    quitting: bool;
}

file_pool: *File_Pool;
// 0 free, 1 taken: guards `file_pool` and its `users`.
file_pool_lock: s32;

// Reads the pool's settings from the environment into the engine.
pool_configure :: (engine: *Engine) {
    engine.file_threads = min(env_number("JAI_IO_FILE_THREADS", DEFAULT_FILE_THREADS), POOL_MAX_THREADS);
    engine.file_inline = env_number("JAI_IO_FILE_INLINE", DEFAULT_FILE_INLINE);
}

// A non-negative integer from the environment, or `fallback`.
env_number :: (name: *u8, fallback: s64) -> s64 {
    scratch: [256] u8;
    text := env_text(name, *scratch);

    if text.count == 0 return fallback;

    value, ok := parse_decimal(text, 0x7fff_ffff);
    if !ok return fallback;

    return value;
}

#if OS == .LINUX || OS == .MACOS {
    pool_lock :: () {
        while true {
            exchanged, _ := atomic_compare_exchange(*file_pool_lock, cast(s32) 0, cast(s32) 1);
            if exchanged break;

            sleep_milliseconds(0);
        }
    }

    pool_unlock :: () {
        atomic_store(*file_pool_lock, cast(s32) 0);
    }

    #if OS == .LINUX {
        c_eventfd :: (count: u32, flags: s32) -> s32 #foreign libc "eventfd";

        EFD_CLOEXEC :: 0x80000;
        EFD_NONBLOCK :: 0x800;
    } else {
        EVFILT_USER :: -10;
        EV_CLEAR :: 0x20;
        NOTE_TRIGGER :: 0x0100_0000;
        // `ident` of the one user event of a kqueue.
        POOL_EVENT :: 0x6a61;
    }

    // Takes the engine's part in the pool, starting the workers if it is the first. Null if the
    // pool cannot be had (the caller then runs the operation inline).
    pool_attach :: (engine: *Engine) -> *Pool_Link {
        if engine.pool return engine.pool;

        pool_lock();

        if !file_pool {
            pool := New(File_Pool);

            Thread.init(*pool.mutex);
            Thread.init(*pool.work);

            for 0..engine.file_threads - 1 {
                worker := *pool.workers[it];

                Thread.thread_init(worker, pool_worker);
                worker.data = pool;

                if Thread.thread_start(worker) pool.started += 1;
                else Thread.thread_deinit(worker);
            }

            if pool.started == 0 {
                Thread.destroy(*pool.mutex);
                Thread.destroy(*pool.work);
                free(pool);

                pool_unlock();

                return null;
            }

            file_pool = pool;
        }

        file_pool.users += 1;

        pool_unlock();

        link := New(Pool_Link);

        Thread.init(*link.mutex);

        if !pool_open_wake(engine, link) {
            Thread.destroy(*link.mutex);
            free(link);
            pool_release();

            return null;
        }

        engine.pool = link;

        return link;
    }

    // One scheduler less; the last one ends the workers.
    pool_release :: () {
        pool_lock();

        pool := file_pool;
        pool.users -= 1;

        if pool.users > 0 {
            pool_unlock();

            return;
        }

        file_pool = null;

        pool_unlock();

        Thread.lock(*pool.mutex);
        pool.quitting = true;
        Thread.unlock(*pool.mutex);

        for 1..pool.started Thread.signal(*pool.work);
        for 0..pool.started - 1 Thread.thread_deinit(*pool.workers[it]);

        Thread.destroy(*pool.mutex);
        Thread.destroy(*pool.work);

        free(pool);
    }

    // Hands the read or write `op` to a worker. False if the pool is not available.
    pool_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) -> bool {
        link := pool_attach(engine);
        if !link return false;

        op.uncancelable = true;
        op.link = link;
        op.next = null;

        link.inflight += 1;
        pool_arm(scheduler, engine, link);

        pool := file_pool;

        Thread.lock(*pool.mutex);
        if pool.tail pool.tail.next = op;
        else pool.head = op;

        pool.tail = op;
        Thread.unlock(*pool.mutex);

        Thread.signal(*pool.work);

        return true;
    }

    // A worker: takes operations off the queue, makes the call, reports to the owner.
    pool_worker :: (thread: *Thread.Thread) -> s64 {
        pool := cast(*File_Pool) thread.data;

        while true {
            Thread.wait_for(*pool.work);

            Thread.lock(*pool.mutex);

            op := pool.head;

            if op {
                pool.head = op.next;
                if !pool.head pool.tail = null;

                op.next = null;
            }

            quitting := pool.quitting;

            Thread.unlock(*pool.mutex);

            if !op {
                if quitting break;

                continue;
            }

            if op.kind == .READ op.result = sys_read(op.handle.fd, op.buffer, op.length, op.offset);
            else op.result = sys_write(op.handle.fd, op.buffer, op.length, op.offset);

            link := op.link;

            Thread.lock(*link.mutex);

            if link.done_tail link.done_tail.next = op;
            else link.done_head = op;

            link.done_tail = op;

            pool_notify(link);
            Thread.unlock(*link.mutex);
        }

        return 0;
    }

    // Finishes what the workers have completed.
    pool_collect :: (scheduler: *Scheduler, engine: *Engine) {
        link := engine.pool;
        if !link return;

        Thread.lock(*link.mutex);

        op := link.done_head;

        link.done_head = null;
        link.done_tail = null;

        Thread.unlock(*link.mutex);

        while op {
            next := op.next;

            op.next = null;
            link.inflight -= 1;

            finish_op(scheduler, op, op.result);

            op = next;
        }
    }

    // Blocks the thread until every operation handed to the pool has been taken back.
    pool_drain :: (scheduler: *Scheduler, engine: *Engine) {
        link := engine.pool;
        if !link return;

        while link.inflight > 0 {
            pool_collect(scheduler, engine);

            if link.inflight > 0 sleep_milliseconds(1);
        }
    }

    // The engine is going away: wait for the workers, release the wake-up, leave the pool.
    pool_detach :: (scheduler: *Scheduler, engine: *Engine) {
        link := engine.pool;
        if !link return;

        pool_drain(scheduler, engine);

        pool_close_wake(scheduler, engine, link);

        // A worker may still be inside its last `unlock`.
        Thread.lock(*link.mutex);
        Thread.unlock(*link.mutex);
        Thread.destroy(*link.mutex);

        free(link);
        engine.pool = null;

        pool_release();
    }

    #if OS == .LINUX {
        pool_open_wake :: (engine: *Engine, link: *Pool_Link) -> bool {
            fd := c_eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK);
            if fd < 0 return false;

            link.wake_fd = fd;

            return true;
        }

        pool_close_wake :: (scheduler: *Scheduler, engine: *Engine, link: *Pool_Link) {
            if link.wake_op engine_cancel(scheduler, link.wake_op);

            poll_close(scheduler, engine, link.wake_fd);
            sys_close(link.wake_fd);
        }

        // Called with the link's mutex held.
        pool_notify :: (link: *Pool_Link) {
            one: u64 = 1;

            c_write(cast(s32) link.wake_fd, *one, 8);
        }

        // Keeps a READ on the eventfd outstanding while operations are out.
        pool_arm :: (scheduler: *Scheduler, engine: *Engine, link: *Pool_Link) {
            while !link.wake_op && link.inflight > 0 {
                op := new_op(engine, .READ, Handle.{ link.wake_fd, .STREAM });

                op.internal = true;
                op.tag = .POOL_WAKE;
                op.buffer = link.wake_buffer.data;
                op.length = 8;

                if engine_submit(scheduler, op) {
                    free_op(engine, op);
                    pool_collect(scheduler, engine);
                } else {
                    link.wake_op = op;
                }
            }
        }

        // The READ on the eventfd completed.
        pool_wake_completed :: (scheduler: *Scheduler, op: *Op) {
            engine := scheduler.engine;
            link := engine.pool;

            link.wake_op = null;

            canceled := op.result == -E_CANCELED;

            free_op(engine, op);

            if canceled return;

            pool_collect(scheduler, engine);
            pool_arm(scheduler, engine, link);
        }
    } else {
        pool_open_wake :: (engine: *Engine, link: *Pool_Link) -> bool {
            link.queue_fd = engine.poll_fd;

            change: Kevent;
            change.ident = POOL_EVENT;
            change.filter = EVFILT_USER;
            change.flags = EV_ADD | EV_CLEAR;

            return c_kevent(engine.poll_fd, *change, 1, null, 0, null) >= 0;
        }

        pool_close_wake :: (scheduler: *Scheduler, engine: *Engine, link: *Pool_Link) {}

        pool_notify :: (link: *Pool_Link) {
            change: Kevent;
            change.ident = POOL_EVENT;
            change.filter = EVFILT_USER;
            change.fflags = NOTE_TRIGGER;

            c_kevent(cast(s32) link.queue_fd, *change, 1, null, 0, null);
        }

        pool_arm :: (scheduler: *Scheduler, engine: *Engine, link: *Pool_Link) {}

        pool_wake_completed :: (scheduler: *Scheduler, op: *Op) {}
    }
} else {
    pool_submit :: (scheduler: *Scheduler, engine: *Engine, op: *Op) -> bool {
        return false;
    }

    pool_collect :: (scheduler: *Scheduler, engine: *Engine) {}
    pool_drain :: (scheduler: *Scheduler, engine: *Engine) {}
    pool_detach :: (scheduler: *Scheduler, engine: *Engine) {}
    pool_wake_completed :: (scheduler: *Scheduler, op: *Op) {}
}
