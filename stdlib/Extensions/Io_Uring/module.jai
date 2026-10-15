// Io_Uring: Linux's io_uring interface, as thin as the kernel makes it. A jaic extension, not
// official Jai: other Jai compilers have no such module, and it may change between jaic releases.
// See docs/stdlib/io-uring.md.
//
//     Io_Uring :: #import "Extensions/Io_Uring";
//
//     ring: Io_Uring.Ring;
//     if Io_Uring.ring_init(*ring, 8) < 0 return;     // the kernel's -errno, or 0
//     defer Io_Uring.ring_deinit(*ring);
//
//     sqe := Io_Uring.get_sqe(*ring);
//     Io_Uring.prep_nop(sqe);
//     sqe.user_data = 42;
//
//     Io_Uring.submit_and_wait(*ring, 1);
//
//     cqe: *Io_Uring.io_uring_cqe;
//     Io_Uring.wait_cqe(*ring, *cqe);                  // cqe.user_data == 42, cqe.res == 0
//     Io_Uring.cqe_seen(*ring, cqe);
//
// Three layers, each usable alone:
//   - bindings.jai: the kernel's constants, enums and structs, GENERATED from linux/io_uring.h by
//     tools/gen_io_uring.py with the kernel's names and layout (the `io_uring_sqe`, `io_uring_cqe`
//     and `io_uring_params` of C), and `#assert`s that jaic lays each one out like C does.
//   - syscalls.jai: io_uring_setup, io_uring_enter and io_uring_register, which return what the
//     kernel returns: a result of 0 or more, or a negative errno.
//   - ring.jai: a Ring that maps the shared memory and keeps the head and tail protocol (canonical
//     sequentially consistent atomics, stronger than acquire/release): get_sqe, submit, peek_cqe,
//     wait_cqe, cqe_seen, and probe_ops to ask which operations this kernel supports.
//
// This is a binding, not an IO API: use Extensions/Io for fibers over files and sockets. Linux
// only (the kernel interface exists nowhere else); the module is a compile error on other
// systems. Whether io_uring works at run time depends on the kernel (5.1 or newer; the rings
// here want 5.6 or newer for probing) and on its policy: containers and seccomp filters often
// refuse the calls, and `kernel.io_uring_disabled` can switch it off. ring_init then returns
// -ENOSYS or -EPERM; programs and tests should treat that as "not available" and carry on.
#assert OS == .LINUX "Extensions/Io_Uring is Linux only: io_uring is a Linux kernel interface";

#load "bindings.jai";
#load "syscalls.jai";
#load "ring.jai";
