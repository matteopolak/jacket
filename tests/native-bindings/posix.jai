P :: #import "POSIX";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    P.set_errno(cast(P.OS_Error_Code) P.ENOENT);
    check(P.errno() == P.ENOENT);
    P.set_errno(cast(P.OS_Error_Code) 0);
    missing := P.open(".work/no-such-binding-test-file", P.O_RDONLY);

    check(missing == -1 && P.errno() == P.ENOENT);
    first, second: P.timespec;

    check(P.clock_gettime(.MONOTONIC, *first) == 0);
    check(P.clock_gettime(.MONOTONIC, *second) == 0);
    check(second.tv_sec > first.tv_sec || (second.tv_sec == first.tv_sec && second.tv_nsec >= first.tv_nsec));
    mutex: P.pthread_mutex_t;

    check(P.pthread_mutex_init(*mutex, null) == 0);
    check(P.pthread_mutex_lock(*mutex) == 0);
    check(P.pthread_mutex_unlock(*mutex) == 0);
    check(P.pthread_mutex_destroy(*mutex) == 0);
    check(P.S_ISDIR(cast(P.mode_t) P.S_IFDIR));
    check(!P.S_ISREG(cast(P.mode_t) P.S_IFDIR));
    check(P.WIFEXITED(17 << 8));
    check(P.WEXITSTATUS(17 << 8) == 17);
    check(P.WIFSIGNALED(9));

    #if OS == .MACOS {
        check(P.WIFCONTINUED(0x137f));
        check(!P.WIFSTOPPED(0x137f));
        status: P.wait_t;

        P.set_w_T_w_Termsig(*status, 9);
        P.set_w_T_w_Coredump(*status, true);
        P.set_w_T_w_Retcode(*status, 17);
        check(P.get_w_T_w_Termsig(status) == 9);
        check(P.get_w_T_w_Coredump(status));
        check(P.get_w_T_w_Retcode(status) == 17);
        P.set_w_T_w_Coredump(*status, false);
        check(!P.get_w_T_w_Coredump(status));
        check(P.get_w_T_w_Termsig(status) == 9 && P.get_w_T_w_Retcode(status) == 17);
    }
}
