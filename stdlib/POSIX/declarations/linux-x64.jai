// Generated from docs/specs/stdlib/modules/posix.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// The declarations for `OS == .LINUX && CPU == .X64`, loaded by declarations.jai.

#scope_file
U128 :: [2] u64;

#if OS == .MACOS || OS == .IOS {
    binding_errno_address :: () -> *s32 #foreign libc "__error";
} else #if OS == .ANDROID {
    binding_errno_address :: () -> *s32 #foreign libc "__errno";
} else {
    binding_errno_address :: () -> *s32 #foreign libc "__errno_location";
}

libc :: #library,system "c"; @native(external)
libdl :: #library,system "dl"; @native(external)
libpthread :: #library,system "pthread"; @native(external)
librt :: #library,system "librt"; @native(external)
#scope_export

mcontext_t :: struct {
    gregs: gregset_t;

    fpregs: fpregset_t;
    __reserved1: [8] u64;
}

sigcontext :: struct {
    r8: u64;
    r9: u64;
    r10: u64;
    r11: u64;
    r12: u64;
    r13: u64;
    r14: u64;
    r15: u64;
    rdi: u64;
    rsi: u64;
    rbp: u64;
    rbx: u64;
    rdx: u64;
    rax: u64;
    rcx: u64;
    rsp: u64;
    rip: u64;
    eflags: u64;
    cs: u16;
    gs: u16;
    fs: u16;
    __pad0: u16;
    err: u64;
    trapno: u64;
    oldmask: u64;
    cr2: u64;
    struct {
        fpstate: *_fpstate;

        #overlay (fpstate) __fpstate_word: u64;
    }

    __reserved1: [8] u64;
}

stat64_t :: struct {
    st_dev: u64;

    st_ino: u64;
    st_nlink: u64;
    st_mode: u32;

    st_uid: u32;
    st_gid: u32;

    __pad0: s32;
    st_rdev: u64;
    st_size: s64;

    st_blksize: s64;
    st_blocks: s64;

    st_atime: timespec;
    st_mtime: timespec;
    st_ctime: timespec;

    __glibc_reserved: [3] s64;
}

stat_t :: struct {
    st_dev: u64;

    st_ino: u64;

    st_nlink: u64;
    st_mode: u32;

    st_uid: u32;
    st_gid: u32;

    __pad0: s32;

    st_rdev: u64;

    st_size: s64;

    st_blksize: s64;

    st_blocks: s64;

    st_atime: timespec;
    st_mtime: timespec;
    st_ctime: timespec;

    __glibc_reserved: [3] s64;
}

ucontext_t :: struct {
    uc_flags: u64;
    uc_link: *ucontext_t;
    uc_stack: stack_t;
    uc_mcontext: mcontext_t;
    uc_sigmask: sigset_t;
    __fpregs_mem: _libc_fpstate;
    __ssp: [4] u64;
}

__jmp_buf :: [8] s64;

__O_DIRECT :: 0x4000;

__O_DIRECTORY :: 0x10000;

__O_NOFOLLOW :: 0x20000;

blksize_t :: s64;

nlink_t :: u64;

SYS__sysctl :: 156;

SYS_accept :: 43;

SYS_accept4 :: 288;

SYS_access :: 21;

SYS_acct :: 163;

SYS_add_key :: 248;

SYS_adjtimex :: 159;

SYS_afs_syscall :: 183;

SYS_alarm :: 37;

SYS_arch_prctl :: 158;

SYS_bind :: 49;

SYS_bpf :: 321;

SYS_brk :: 12;

SYS_capget :: 125;

SYS_capset :: 126;

SYS_chdir :: 80;

SYS_chmod :: 90;

SYS_chown :: 92;

SYS_chroot :: 161;

SYS_clock_adjtime :: 305;

SYS_clock_getres :: 229;

SYS_clock_gettime :: 228;

SYS_clock_nanosleep :: 230;

SYS_clock_settime :: 227;

SYS_clone :: 56;

SYS_clone3 :: 435;

SYS_close :: 3;

SYS_connect :: 42;

SYS_copy_file_range :: 326;

SYS_creat :: 85;

SYS_create_module :: 174;

SYS_delete_module :: 176;

SYS_dup :: 32;

SYS_dup2 :: 33;

SYS_dup3 :: 292;

SYS_epoll_create :: 213;

SYS_epoll_create1 :: 291;

SYS_epoll_ctl :: 233;

SYS_epoll_ctl_old :: 214;

SYS_epoll_pwait :: 281;

SYS_epoll_wait :: 232;

SYS_epoll_wait_old :: 215;

SYS_eventfd :: 284;

SYS_eventfd2 :: 290;

SYS_execve :: 59;

SYS_execveat :: 322;

SYS_exit :: 60;

SYS_exit_group :: 231;

SYS_faccessat :: 269;

SYS_fadvise64 :: 221;

SYS_fallocate :: 285;

SYS_fanotify_init :: 300;

SYS_fanotify_mark :: 301;

SYS_fchdir :: 81;

SYS_fchmod :: 91;

SYS_fchmodat :: 268;

SYS_fchown :: 93;

SYS_fchownat :: 260;

SYS_fcntl :: 72;

SYS_fdatasync :: 75;

SYS_fgetxattr :: 193;

SYS_finit_module :: 313;

SYS_flistxattr :: 196;

SYS_flock :: 73;

SYS_fork :: 57;

SYS_fremovexattr :: 199;

SYS_fsconfig :: 431;

SYS_fsetxattr :: 190;

SYS_fsmount :: 432;

SYS_fsopen :: 430;

SYS_fspick :: 433;

SYS_fstat :: 5;

SYS_fstatfs :: 138;

SYS_fsync :: 74;

SYS_ftruncate :: 77;

SYS_futex :: 202;

SYS_futimesat :: 261;

SYS_get_kernel_syms :: 177;

SYS_get_mempolicy :: 239;

SYS_get_robust_list :: 274;

SYS_get_thread_area :: 211;

SYS_getcpu :: 309;

SYS_getcwd :: 79;

SYS_getdents :: 78;

SYS_getdents64 :: 217;

SYS_getegid :: 108;

SYS_geteuid :: 107;

SYS_getgid :: 104;

SYS_getgroups :: 115;

SYS_getitimer :: 36;

SYS_getpeername :: 52;

SYS_getpgid :: 121;

SYS_getpgrp :: 111;

SYS_getpid :: 39;

SYS_getpmsg :: 181;

SYS_getppid :: 110;

SYS_getpriority :: 140;

SYS_getrandom :: 318;

SYS_getresgid :: 120;

SYS_getresuid :: 118;

SYS_getrlimit :: 97;

SYS_getrusage :: 98;

SYS_getsid :: 124;

SYS_getsockname :: 51;

SYS_getsockopt :: 55;

SYS_gettid :: 186;

SYS_gettimeofday :: 96;

SYS_getuid :: 102;

SYS_getxattr :: 191;

SYS_init_module :: 175;

SYS_inotify_add_watch :: 254;

SYS_inotify_init :: 253;

SYS_inotify_init1 :: 294;

SYS_inotify_rm_watch :: 255;

SYS_io_cancel :: 210;

SYS_io_destroy :: 207;

SYS_io_getevents :: 208;

SYS_io_pgetevents :: 333;

SYS_io_setup :: 206;

SYS_io_submit :: 209;

SYS_io_uring_enter :: 426;

SYS_io_uring_register :: 427;

SYS_io_uring_setup :: 425;

SYS_ioctl :: 16;

SYS_ioperm :: 173;

SYS_iopl :: 172;

SYS_ioprio_get :: 252;

SYS_ioprio_set :: 251;

SYS_kcmp :: 312;

SYS_kexec_file_load :: 320;

SYS_kexec_load :: 246;

SYS_keyctl :: 250;

SYS_kill :: 62;

SYS_lchown :: 94;

SYS_lgetxattr :: 192;

SYS_link :: 86;

SYS_linkat :: 265;

SYS_listen :: 50;

SYS_listxattr :: 194;

SYS_llistxattr :: 195;

SYS_lookup_dcookie :: 212;

SYS_lremovexattr :: 198;

SYS_lseek :: 8;

SYS_lsetxattr :: 189;

SYS_lstat :: 6;

SYS_madvise :: 28;

SYS_mbind :: 237;

SYS_membarrier :: 324;

SYS_memfd_create :: 319;

SYS_migrate_pages :: 256;

SYS_mincore :: 27;

SYS_mkdir :: 83;

SYS_mkdirat :: 258;

SYS_mknod :: 133;

SYS_mknodat :: 259;

SYS_mlock :: 149;

SYS_mlock2 :: 325;

SYS_mlockall :: 151;

SYS_mmap :: 9;

SYS_modify_ldt :: 154;

SYS_mount :: 165;

SYS_move_mount :: 429;

SYS_move_pages :: 279;

SYS_mprotect :: 10;

SYS_mq_getsetattr :: 245;

SYS_mq_notify :: 244;

SYS_mq_open :: 240;

SYS_mq_timedreceive :: 243;

SYS_mq_timedsend :: 242;

SYS_mq_unlink :: 241;

SYS_mremap :: 25;

SYS_msgctl :: 71;

SYS_msgget :: 68;

SYS_msgrcv :: 70;

SYS_msgsnd :: 69;

SYS_msync :: 26;

SYS_munlock :: 150;

SYS_munlockall :: 152;

SYS_munmap :: 11;

SYS_name_to_handle_at :: 303;

SYS_nanosleep :: 35;

SYS_newfstatat :: 262;

SYS_nfsservctl :: 180;

SYS_open :: 2;

SYS_open_by_handle_at :: 304;

SYS_open_tree :: 428;

SYS_openat :: 257;

SYS_pause :: 34;

SYS_perf_event_open :: 298;

SYS_personality :: 135;

SYS_pidfd_open :: 434;

SYS_pidfd_send_signal :: 424;

SYS_pipe :: 22;

SYS_pipe2 :: 293;

SYS_pivot_root :: 155;

SYS_pkey_alloc :: 330;

SYS_pkey_free :: 331;

SYS_pkey_mprotect :: 329;

SYS_poll :: 7;

SYS_ppoll :: 271;

SYS_prctl :: 157;

SYS_pread64 :: 17;

SYS_preadv :: 295;

SYS_preadv2 :: 327;

SYS_prlimit64 :: 302;

SYS_process_vm_readv :: 310;

SYS_process_vm_writev :: 311;

SYS_pselect6 :: 270;

SYS_ptrace :: 101;

SYS_putpmsg :: 182;

SYS_pwrite64 :: 18;

SYS_pwritev :: 296;

SYS_pwritev2 :: 328;

SYS_query_module :: 178;

SYS_quotactl :: 179;

SYS_read :: 0;

SYS_readahead :: 187;

SYS_readlink :: 89;

SYS_readlinkat :: 267;

SYS_readv :: 19;

SYS_reboot :: 169;

SYS_recvfrom :: 45;

SYS_recvmmsg :: 299;

SYS_recvmsg :: 47;

SYS_remap_file_pages :: 216;

SYS_removexattr :: 197;

SYS_rename :: 82;

SYS_renameat :: 264;

SYS_renameat2 :: 316;

SYS_request_key :: 249;

SYS_restart_syscall :: 219;

SYS_rmdir :: 84;

SYS_rseq :: 334;

SYS_rt_sigaction :: 13;

SYS_rt_sigpending :: 127;

SYS_rt_sigprocmask :: 14;

SYS_rt_sigqueueinfo :: 129;

SYS_rt_sigreturn :: 15;

SYS_rt_sigsuspend :: 130;

SYS_rt_sigtimedwait :: 128;

SYS_rt_tgsigqueueinfo :: 297;

SYS_sched_get_priority_max :: 146;

SYS_sched_get_priority_min :: 147;

SYS_sched_getaffinity :: 204;

SYS_sched_getattr :: 315;

SYS_sched_getparam :: 143;

SYS_sched_getscheduler :: 145;

SYS_sched_rr_get_interval :: 148;

SYS_sched_setaffinity :: 203;

SYS_sched_setattr :: 314;

SYS_sched_setparam :: 142;

SYS_sched_setscheduler :: 144;

SYS_sched_yield :: 24;

SYS_seccomp :: 317;

SYS_security :: 185;

SYS_select :: 23;

SYS_semctl :: 66;

SYS_semget :: 64;

SYS_semop :: 65;

SYS_semtimedop :: 220;

SYS_sendfile :: 40;

SYS_sendmmsg :: 307;

SYS_sendmsg :: 46;

SYS_sendto :: 44;

SYS_set_mempolicy :: 238;

SYS_set_robust_list :: 273;

SYS_set_thread_area :: 205;

SYS_set_tid_address :: 218;

SYS_setdomainname :: 171;

SYS_setfsgid :: 123;

SYS_setfsuid :: 122;

SYS_setgid :: 106;

SYS_setgroups :: 116;

SYS_sethostname :: 170;

SYS_setitimer :: 38;

SYS_setns :: 308;

SYS_setpgid :: 109;

SYS_setpriority :: 141;

SYS_setregid :: 114;

SYS_setresgid :: 119;

SYS_setresuid :: 117;

SYS_setreuid :: 113;

SYS_setrlimit :: 160;

SYS_setsid :: 112;

SYS_setsockopt :: 54;

SYS_settimeofday :: 164;

SYS_setuid :: 105;

SYS_setxattr :: 188;

SYS_shmat :: 30;

SYS_shmctl :: 31;

SYS_shmdt :: 67;

SYS_shmget :: 29;

SYS_shutdown :: 48;

SYS_sigaltstack :: 131;

SYS_signalfd :: 282;

SYS_signalfd4 :: 289;

SYS_socket :: 41;

SYS_socketpair :: 53;

SYS_splice :: 275;

SYS_stat :: 4;

SYS_statfs :: 137;

SYS_statx :: 332;

SYS_swapoff :: 168;

SYS_swapon :: 167;

SYS_symlink :: 88;

SYS_symlinkat :: 266;

SYS_sync :: 162;

SYS_sync_file_range :: 277;

SYS_syncfs :: 306;

SYS_sysfs :: 139;

SYS_sysinfo :: 99;

SYS_syslog :: 103;

SYS_tee :: 276;

SYS_tgkill :: 234;

SYS_time :: 201;

SYS_timer_create :: 222;

SYS_timer_delete :: 226;

SYS_timer_getoverrun :: 225;

SYS_timer_gettime :: 224;

SYS_timer_settime :: 223;

SYS_timerfd_create :: 283;

SYS_timerfd_gettime :: 287;

SYS_timerfd_settime :: 286;

SYS_times :: 100;

SYS_tkill :: 200;

SYS_truncate :: 76;

SYS_tuxcall :: 184;

SYS_umask :: 95;

SYS_umount2 :: 166;

SYS_uname :: 63;

SYS_unlink :: 87;

SYS_unlinkat :: 263;

SYS_unshare :: 272;

SYS_uselib :: 134;

SYS_userfaultfd :: 323;

SYS_ustat :: 136;

SYS_utime :: 132;

SYS_utimensat :: 280;

SYS_utimes :: 235;

SYS_vfork :: 58;

SYS_vhangup :: 153;

SYS_vmsplice :: 278;

SYS_vserver :: 236;

SYS_wait4 :: 61;

SYS_waitid :: 247;

SYS_write :: 1;

SYS_writev :: 20;
