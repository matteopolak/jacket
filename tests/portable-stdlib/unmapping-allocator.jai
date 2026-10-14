#import "Basic";
U :: #import "Unmapping_Allocator";
P :: #import "POSIX";
T :: #import "Thread";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

assert_write_faults :: (memory: *u8) {
    child := P.fork();

    assert(child >= 0);

    if child == 0 {
        memset(memory, 1, 1);
        P._exit(0);
    }

    status: s32;
    waited := P.waitpid(child, *status, 0);

    while waited < 0 && P.errno() == P.EINTR waited = P.waitpid(child, *status, 0);

    assert(waited == child && P.WIFSIGNALED(status));
    signal := P.WTERMSIG(status);

    assert(signal == P.SIGSEGV || signal == P.SIGBUS);
}

main :: () {
    allocator := U.get_unmapping_allocator();
    wrapper := cast(*U.Unmapping_Allocator) allocator.data;
    memory := cast(*u8) allocator.proc(.ALLOCATE, 37, 0, null, allocator.data);

    assert(memory != null && wrapper.table.count == 1);
    page := P.sysconf(P._SC_PAGESIZE);

    assert(page > 0 && cast(u64) (memory + 37) % cast(u64) page == 0);
    memset(memory, 42, 37);
    assert_write_faults(memory + 37);
    previous := context.logger;

    context.logger = capture;
    failed := allocator.proc(.RESIZE, 0x7fffffffffffffff, 37, memory, allocator.data);

    context.logger = previous;
    assert(failed == null && errors > 0 && wrapper.table.count == 1 && memory[0] == 42);
    memory[0] = 19;
    grown := cast(*u8) allocator.proc(.RESIZE, 101, 37, memory, allocator.data);

    assert(grown != null && grown != memory && wrapper.table.count == 1 && grown[0] == 19 && grown[36] == 42);
    assert_write_faults(memory);
    allocator.proc(.FREE, 0, 0, grown, allocator.data);
    assert(wrapper.table.count == 0 && grown[0] == 19);
    assert_write_faults(grown);
    threads: [4] T.Thread;
    for *thread: threads {
        thread.data = *allocator;
        assert(T.thread_init(thread, concurrent_allocations));
        T.thread_start(thread);
    }

    for *thread: threads {
        assert(T.thread_is_done(thread, 5000));
        T.thread_deinit(thread);
    }

    assert(wrapper.table.count == 0);
    U.deinit_unmapping_allocator(allocator);
    print("ok\n");
}

concurrent_allocations :: (thread: *T.Thread) -> s64 {
    allocator := (cast(*Allocator) thread.data).*;

    for 0..7 {
        memory := cast(*u8) allocator.proc(.ALLOCATE, 32, 0, null, allocator.data);
        assert(memory != null);
        memset(memory, 17, 32);
        allocator.proc(.FREE, 0, 0, memory, allocator.data);
    }

    return 0;
}
