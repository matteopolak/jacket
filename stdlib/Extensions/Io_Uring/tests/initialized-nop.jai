// test: only(.LINUX)
// Source control only on this worker: Linux io_uring syscalls must not run on Mac.
// A runtime consumer must initialize successfully before submission/queue reads. Where the kernel
// refuses to give a ring at all (qemu-user has no io_uring, a seccomp filter, kernel.io_uring_disabled),
// the test says so and exits 0.
#import "Basic";
Uring :: #import "Extensions/Io_Uring";

main :: () {
    ring: Uring.Ring;
    initialized := Uring.ring_init(*ring, 8);

    ENOMEM :: 12;
    EACCES :: 13;
    EPERM :: 1;
    ENOSYS :: 38;

    if initialized == -ENOSYS || initialized == -EPERM || initialized == -EACCES || initialized == -ENOMEM {
        print("skipped: io_uring is not available here (ring_init returned %)\n", initialized);

        return;
    }

    if initialized < 0 {
        print("ring_init=%\n", initialized);
        exit(1);
    }

    defer Uring.ring_deinit(*ring);

    sqe := Uring.get_sqe(*ring);

    assert(sqe != null);
    Uring.prep_nop(sqe);
    sqe.user_data = 42;
    submitted := Uring.submit_and_wait(*ring, 1);

    assert(submitted == 1);

    cqe: *Uring.io_uring_cqe;
    waited := Uring.wait_cqe(*ring, *cqe);

    assert(waited == 0);
    assert(cqe.user_data == 42 && cqe.res == 0);
    Uring.cqe_seen(*ring, cqe);
}
