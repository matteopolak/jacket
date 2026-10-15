// Execute the actual userspace protocol on caller-owned memory on this host.
// No kernel call, substituted syscall, or fabricated kernel result is involved.
// Direct source loads test portable queue operations without changing the Linux
// facade guard. This is not evidence of Linux mapping, syscall or kernel behavior.
#import "Basic";
Thread :: #import "Thread";

#load "../../stdlib/Extensions/Io_Uring/bindings.jai";
#load "../../stdlib/Extensions/Io_Uring/syscalls.jai";
#load "../../stdlib/Extensions/Io_Uring/ring.jai";

exercise :: (with_array: bool, wide: bool, initial: u32) {
    head := initial;
    tail := initial;
    completion_head := initial;
    completion_tail := initial;
    flags: u32;
    indexes: [4] u32 = .[99, 99, 99, 99];
    submissions: [64] u64;
    completions: [16] u64;

    ring: Ring;
    ring.sq_entries = 4;
    ring.sq_mask = 3;
    ring.sq_head = *head;
    ring.sq_tail = *tail;
    ring.sq_flags = *flags;
    ring.sq_array = ifx with_array then indexes.data else null;
    ring.sqes = cast(*u8) submissions.data;
    ring.sqe_bytes = ifx wide then 128 else 64;
    ring.sqe_head = initial;
    ring.sqe_tail = initial;
    ring.cq_entries = 4;
    ring.cq_mask = 3;
    ring.cq_head = *completion_head;
    ring.cq_tail = *completion_tail;
    ring.cqes = cast(*u8) completions.data;
    ring.cqe_bytes = ifx wide then 32 else 16;

    // Multiple laps, including the u32 index rollover, retain mask/stride rules.
    for lap: 0..31 {
        memset(submissions.data, 0xa5, size_of(submissions));
        start := head;

        for index: 0..3 {
            sqe := get_sqe(*ring);
            expected := ring.sqes + ((start + cast(u32) index) & 3) * ring.sqe_bytes;

            assert(cast(*u8) sqe == expected);

            for byte: 0..cast(s64) ring.sqe_bytes - 1 assert(expected[byte] == 0);

            prep_nop(sqe);
            sqe.user_data = cast(u64) (lap * 4 + index + 1);
            assert(sq_pending(*ring) == cast(u32) index + 1);
            assert(tail == start, "reservation must not publish entries");
        }

        assert(get_sqe(*ring) == null, "full SQ must not overwrite an entry");
        assert(head == start, "only the kernel actor owns SQ head");
        assert(publish_sqes(*ring) == 4);
        assert(tail == start + 4 && sq_pending(*ring) == 0);
        assert(publish_sqes(*ring) == 4, "publication without new entries is idempotent");

        for index: 0..3 {
            position := (start + cast(u32) index) & 3;
            if with_array assert(indexes[position] == position);
            else assert(indexes[index] == 99, "NO_SQARRAY must not write an index array");

            sqe := cast(*io_uring_sqe) (ring.sqes + position * ring.sqe_bytes);

            assert(sqe.user_data == cast(u64) (lap * 4 + index + 1));
        }

        // The test's kernel actor takes two entries. Userspace sees two slots.
        atomic_store(*head, start + 2);
        for 1..2 assert(get_sqe(*ring) != null);

        assert(get_sqe(*ring) == null);
        assert(publish_sqes(*ring) == 4);
        atomic_store(*head, tail);
        assert(publish_sqes(*ring) == 0);

        // Populate completions before publishing the tail, then consume in order.
        assert(cq_ready(*ring) == 0);
        cq_start := completion_tail;

        for index: 0..3 {
            position := (cq_start + cast(u32) index) & 3;
            cqe := cast(*io_uring_cqe) (ring.cqes + position * ring.cqe_bytes);
            cqe.user_data = cast(u64) index + 100;
            cqe.res = cast(s32) index;
        }

        assert(cq_ready(*ring) == 0, "unpublished entries must remain invisible");
        atomic_store(*completion_tail, cq_start + 4);
        assert(cq_ready(*ring) == 4);

        for index: 0..3 {
            cqe := cast(*io_uring_cqe) (ring.cqes + (completion_head & 3) * ring.cqe_bytes);

            assert(cqe.user_data == cast(u64) index + 100 && cqe.res == cast(s32) index);
            cqe_seen(*ring, cqe);
            assert(cq_ready(*ring) == 3 - cast(u32) index);
            assert(completion_tail == cq_start + 4, "only the kernel actor owns CQ tail");
        }
    }
}

// A second real host thread owns the kernel-side indexes in caller memory.
// It does not execute or substitute an io_uring syscall.
peer :: (thread: *Thread.Thread) -> s64 {
    ring := cast(*Ring) thread.data;

    for sequence: 1..1000 {
        head := ring.sq_head.*;
        while atomic_load(ring.sq_tail) == head sleep_milliseconds(0);

        index := ring.sq_array[head & ring.sq_mask];
        sqe := cast(*io_uring_sqe) (ring.sqes + index * ring.sqe_bytes);

        assert(sqe.user_data == cast(u64) sequence);
        atomic_store(ring.sq_head, head + 1);

        tail := ring.cq_tail.*;
        while tail - atomic_load(ring.cq_head) >= ring.cq_entries sleep_milliseconds(0);

        cqe := cast(*io_uring_cqe) (ring.cqes + (tail & ring.cq_mask) * ring.cqe_bytes);
        cqe.user_data = cast(u64) sequence;
        cqe.res = cast(s32) sequence;
        atomic_store(ring.cq_tail, tail + 1);
    }

    return 0;
}

threaded :: () {
    head: u32;
    tail: u32;
    completion_head: u32;
    completion_tail: u32;
    indexes: [4] u32;
    submissions: [4] io_uring_sqe;
    completions: [4] io_uring_cqe;

    ring: Ring;
    ring.sq_entries = 4;
    ring.sq_mask = 3;
    ring.sq_head = *head;
    ring.sq_tail = *tail;
    ring.sq_array = indexes.data;
    ring.sqes = cast(*u8) submissions.data;
    ring.sqe_bytes = size_of(io_uring_sqe);
    ring.cq_entries = 4;
    ring.cq_mask = 3;
    ring.cq_head = *completion_head;
    ring.cq_tail = *completion_tail;
    ring.cqes = cast(*u8) completions.data;
    ring.cqe_bytes = size_of(io_uring_cqe);

    thread: Thread.Thread;

    assert(Thread.thread_init(*thread, peer));
    thread.data = *ring;
    Thread.thread_start(*thread);

    for sequence: 1..1000 {
        sqe := get_sqe(*ring);

        assert(sqe != null);
        prep_nop(sqe);
        sqe.user_data = cast(u64) sequence;
        publish_sqes(*ring);

        while cq_ready(*ring) == 0 sleep_milliseconds(0);

        cqe := *completions[completion_head & 3];

        assert(cqe.user_data == cast(u64) sequence && cqe.res == cast(s32) sequence);
        cqe_seen(*ring, cqe);
    }

    assert(Thread.thread_is_done(*thread, -1));
    Thread.thread_deinit(*thread);
    assert(head == 1000 && tail == 1000 && completion_head == 1000 && completion_tail == 1000);
}

main :: () {
    for with_array: 0..1 {
        for wide: 0..1 {
            exercise(with_array != 0, wide != 0, 0);
            exercise(with_array != 0, wide != 0, 0xfffffffe);
        }
    }

    threaded();
    print("ok\n");
}
