// Independently authored Linux UAPI calls through the trusted libc syscall entry.
io_uring_setup :: (entries: u32, p: *io_uring_params) -> s32 {
    return cast(s32) syscall(425, entries, p);
}
io_uring_enter :: (fd: s32, to_submit: u32, min_complete: u32, flags: u32) -> s32 {
    return cast(s32) syscall(426, fd, to_submit, min_complete, flags, null, 8);
}
io_uring_enter :: (fd: s32, to_submit: u32, min_complete: u32, flags: u32, sig: *sigset_t) -> s32 {
    return cast(s32) syscall(426, fd, to_submit, min_complete, flags, sig, 8);
}
io_uring_enter :: (fd: s32, to_submit: u32, min_complete: u32, flags: u32, arg: *io_uring_getevents_arg) -> s32 {
    extended := flags | cast(u32) IORING_ENTER_EXT_ARG;
    return cast(s32) syscall(426, fd, to_submit, min_complete, extended, arg, size_of(io_uring_getevents_arg));
}
io_uring_register :: (fd: s32, opcode: u32, arg: *void, nr_args: u32) -> s32 {
    return cast(s32) syscall(427, fd, opcode, arg, nr_args);
}
