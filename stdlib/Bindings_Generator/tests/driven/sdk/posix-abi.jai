P :: #import "POSIX";

probe :: #library,system "posix_abi_probe";
native_size :: (record: u32) -> u64 #foreign probe "binding_posix_size";
native_offset :: (field: u32) -> u64 #foreign probe "binding_posix_offset";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(size_of(P.timeval) == native_size(0));
    check(size_of(P.timespec) == native_size(1));
    check(size_of(P.itimerval) == native_size(2));
    check(size_of(P.iovec) == native_size(3));
    check(size_of(P.pollfd) == native_size(4));
    check(size_of(P.ipc_perm) == native_size(5));
    check(size_of(P.stack_t) == native_size(6));
    check(offset_of(P.timeval, "tv_usec") == native_offset(0));
    check(offset_of(P.timespec, "tv_nsec") == native_offset(1));
    check(offset_of(P.itimerval, "it_value") == native_offset(2));
    check(offset_of(P.iovec, "iov_len") == native_offset(3));
    check(offset_of(P.pollfd, "revents") == native_offset(4));
    check(offset_of(P.ipc_perm, "mode") == native_offset(5));
    check(offset_of(P.stack_t, "ss_flags") == native_offset(6));
    timer: P.itimerval;

    check(P.getitimer(0, *timer) == 0);
    check(timer.it_value.tv_usec >= 0 && timer.it_value.tv_usec < 1_000_000);
    check(timer.it_interval.tv_usec >= 0 && timer.it_interval.tv_usec < 1_000_000);
}
