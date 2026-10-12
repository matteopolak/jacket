// The three io_uring system calls. Each returns what the kernel returns: 0 or more on success and
// a negative errno on failure, never -1 with an errno to read afterwards. The numbers are the same
// on every Linux CPU that has io_uring (x86-64 and arm64 share the generic range).
SYS_io_uring_setup :: 425;
SYS_io_uring_enter :: 426;
SYS_io_uring_register :: 427;

// Creates a ring of at least `entries` submission entries. `params` goes in (flags, and the CPU and
// idle time of an SQPOLL thread) and comes back filled in: the real sizes, `features` and the
// offsets to map. Returns the ring's file descriptor.
io_uring_setup :: (entries: u32, params: *io_uring_params) -> s32 {
    return kernel_result(raw_syscall(SYS_io_uring_setup, entries, cast(s64) params, 0, 0, 0, 0));
}

// Submits `to_submit` entries and/or waits for `min_complete` completions, depending on `flags`
// (IORING_ENTER_*). `argument` is a signal mask, or an io_uring_getevents_arg with
// IORING_ENTER_EXT_ARG, and `argument_bytes` its size (8 for a mask).
io_uring_enter :: (fd: s32, to_submit: u32, min_complete: u32, flags: u32, argument: *void, argument_bytes: u64) -> s32 {
    return kernel_result(raw_syscall(SYS_io_uring_enter, fd, to_submit, min_complete, flags, cast(s64) argument, cast(s64) argument_bytes));
}

// Registers resources with the ring (IORING_REGISTER_*, see io_uring_register_op).
io_uring_register :: (fd: s32, opcode: u32, argument: *void, count: u32) -> s32 {
    return kernel_result(raw_syscall(SYS_io_uring_register, fd, opcode, cast(s64) argument, count, 0, 0));
}

// errno values this module returns or tests for. They match on every Linux CPU.
ERR_PERM :: 1;
ERR_INTR :: 4;
ERR_AGAIN :: 11;
ERR_NOMEM :: 12;
ERR_INVAL :: 22;
ERR_NOSYS :: 38;
ERR_TIME :: 62;
ERR_OPNOTSUPP :: 95;

#scope_file

libc :: #system_library "libc";

// libc's variadic syscall() called with six fixed arguments: the same registers on x86-64 and arm64.
raw_syscall :: (number: s64, a1: s64, a2: s64, a3: s64, a4: s64, a5: s64, a6: s64) -> s64 #foreign libc "syscall";
errno_location :: () -> *s32 #foreign libc "__errno_location";

kernel_result :: (result: s64) -> s32 {
    if result == -1 return -errno_location().*;

    return cast(s32) result;
}
